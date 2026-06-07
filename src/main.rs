#![allow(clippy::needless_range_loop)]

mod cube;
mod search;

use cube::{Cube, Move, QUARTER_MOVES};
use rand::Rng;

#[derive(serde::Serialize, serde::Deserialize)]
struct NnueRegressor {
    w1: Vec<Vec<f32>>,
    b1: Vec<f32>,
    w2: Vec<Vec<f32>>,
    b2: Vec<f32>,
    w3: Vec<f32>,
    b3: f32,
    m: usize,
}

#[target_feature(enable = "avx512f")]
unsafe fn sum_row_avx512(accumulator: &mut [f32], weights: &[f32]) {
    let chunks = accumulator.len() / 16;
    for i in 0..chunks {
        unsafe {
            let ptr_a = accumulator.as_mut_ptr().add(i * 16);
            let ptr_w = weights.as_ptr().add(i * 16);
            let v_a = std::arch::x86_64::_mm512_loadu_ps(ptr_a);
            let v_w = std::arch::x86_64::_mm512_loadu_ps(ptr_w);
            let v_res = std::arch::x86_64::_mm512_add_ps(v_a, v_w);
            std::arch::x86_64::_mm512_storeu_ps(ptr_a, v_res);
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn relu_avx512(accumulator: &mut [f32]) {
    let chunks = accumulator.len() / 16;
    unsafe {
        let v_zero = std::arch::x86_64::_mm512_setzero_ps();
        for i in 0..chunks {
            let ptr_a = accumulator.as_mut_ptr().add(i * 16);
            let v_a = std::arch::x86_64::_mm512_loadu_ps(ptr_a);
            let v_res = std::arch::x86_64::_mm512_max_ps(v_a, v_zero);
            std::arch::x86_64::_mm512_storeu_ps(ptr_a, v_res);
        }
    }
}

impl NnueRegressor {
    fn new(m: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut init_he = |in_dim: usize| -> f32 {
            let std_dev = (2.0 / in_dim as f32).sqrt();
            rng.gen_range(-std_dev..std_dev)
        };
        let mut w1 = vec![vec![0.0; m]; 480];
        for i in 0..480 {
            for j in 0..m {
                w1[i][j] = init_he(480);
            }
        }
        let b1 = vec![0.0; m];
        let mut w2 = vec![vec![0.0; 128]; m];
        for i in 0..m {
            for j in 0..128 {
                w2[i][j] = init_he(m);
            }
        }
        let b2 = vec![0.0; 128];
        let mut w3 = vec![0.0; 128];
        for i in 0..128 {
            w3[i] = init_he(128);
        }
        let b3 = 0.0;
        Self {
            w1,
            b1,
            w2,
            b2,
            w3,
            b3,
            m,
        }
    }

    fn forward(&self, active_features: &[usize]) -> (Vec<f32>, Vec<f32>, f32) {
        let mut h1 = self.b1.clone();
        unsafe {
            for &idx in active_features {
                if idx < 480 {
                    sum_row_avx512(&mut h1, &self.w1[idx]);
                }
            }
            relu_avx512(&mut h1);
        }

        let mut h2 = self.b2.clone();
        for i in 0..self.m {
            let val_h1 = h1[i];
            if val_h1 > 0.0 {
                for j in 0..128 {
                    h2[j] += val_h1 * self.w2[i][j];
                }
            }
        }
        for val in &mut h2 {
            if *val < 0.0 {
                *val = 0.0;
            }
        }

        let mut out = self.b3;
        for i in 0..128 {
            out += h2[i] * self.w3[i];
        }
        (h1, h2, out)
    }

    fn backward(
        &mut self,
        active_features: &[usize],
        h1: &[f32],
        h2: &[f32],
        pred: f32,
        target: f32,
        lr: f32,
    ) {
        let d_out = 2.0 * (pred - target);
        let mut dw3 = vec![0.0; 128];
        let mut dh2 = vec![0.0; 128];
        for i in 0..128 {
            dw3[i] = d_out * h2[i];
            dh2[i] = d_out * self.w3[i];
            if h2[i] <= 0.0 {
                dh2[i] = 0.0;
            }
        }
        let db3 = d_out;
        let mut dw2 = vec![vec![0.0; 128]; self.m];
        let mut dh1 = vec![0.0; self.m];
        for i in 0..self.m {
            for j in 0..128 {
                dw2[i][j] = dh2[j] * h1[i];
                dh1[i] += dh2[j] * self.w2[i][j];
            }
            if h1[i] <= 0.0 {
                dh1[i] = 0.0;
            }
        }
        let db2 = dh2;
        let mut dw1 = vec![vec![0.0; self.m]; 480];
        for &idx in active_features {
            if idx < 480 {
                for j in 0..self.m {
                    dw1[idx][j] += dh1[j];
                }
            }
        }
        let db1 = dh1;
        for i in 0..128 {
            self.w3[i] -= lr * dw3[i];
        }
        self.b3 -= lr * db3;
        for i in 0..self.m {
            for j in 0..128 {
                self.w2[i][j] -= lr * dw2[i][j];
            }
        }
        for j in 0..128 {
            self.b2[j] -= lr * db2[j];
        }
        for &idx in active_features {
            if idx < 480 {
                for j in 0..self.m {
                    self.w1[idx][j] -= lr * dw1[idx][j];
                }
            }
        }
        for j in 0..self.m {
            self.b1[j] -= lr * db1[j];
        }
    }
}

fn main() {
    println!("NNUE Regressor initialized.");
}
