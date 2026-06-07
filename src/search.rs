use crate::cube::Cube;
use serde::Deserialize;

#[derive(Deserialize)]
struct HeuristicModelJson {
    w1: Vec<Vec<f32>>,
    b1: Vec<f32>,
    w2: Vec<Vec<f32>>,
    b2: Vec<f32>,
    w3: Vec<f32>,
    b3: f32,
}

pub struct HeuristicModel {
    pub w1: Vec<f32>,
    pub b1: Vec<f32>,
    pub w2: Vec<f32>,
    pub b2: Vec<f32>,
    pub w3: Vec<f32>,
    pub b3: f32,
}

#[target_feature(enable = "avx512f")]
unsafe fn sum_row_avx512(acc: *mut f32, w: *const f32) {
    unsafe {
        for i in 0..16 {
            let pa = acc.add(i * 16);
            let pw = w.add(i * 16);
            std::arch::x86_64::_mm512_storeu_ps(
                pa,
                std::arch::x86_64::_mm512_add_ps(
                    std::arch::x86_64::_mm512_loadu_ps(pa),
                    std::arch::x86_64::_mm512_loadu_ps(pw),
                ),
            );
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn fmadd_row_avx512(acc: *mut f32, w: *const f32, factor: f32) {
    unsafe {
        let vf = std::arch::x86_64::_mm512_set1_ps(factor);
        for i in 0..8 {
            let pa = acc.add(i * 16);
            let pw = w.add(i * 16);
            std::arch::x86_64::_mm512_storeu_ps(
                pa,
                std::arch::x86_64::_mm512_fmadd_ps(
                    vf,
                    std::arch::x86_64::_mm512_loadu_ps(pw),
                    std::arch::x86_64::_mm512_loadu_ps(pa),
                ),
            );
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn relu_avx512<const N: usize>(acc: *mut f32) {
    unsafe {
        let z = std::arch::x86_64::_mm512_setzero_ps();
        for i in 0..N {
            let p = acc.add(i * 16);
            std::arch::x86_64::_mm512_storeu_ps(
                p,
                std::arch::x86_64::_mm512_max_ps(std::arch::x86_64::_mm512_loadu_ps(p), z),
            );
        }
    }
}

#[target_feature(enable = "avx512f")]
unsafe fn dot128_avx512(a: *const f32, b: *const f32) -> f32 {
    unsafe {
        let mut s = std::arch::x86_64::_mm512_setzero_ps();
        for i in 0..8 {
            s = std::arch::x86_64::_mm512_fmadd_ps(
                std::arch::x86_64::_mm512_loadu_ps(a.add(i * 16)),
                std::arch::x86_64::_mm512_loadu_ps(b.add(i * 16)),
                s,
            );
        }
        std::arch::x86_64::_mm512_reduce_add_ps(s)
    }
}

impl HeuristicModel {
    pub fn load() -> Result<Self, String> {
        let data = std::fs::read_to_string("nnue_model_weights.json")
            .map_err(|e| format!("Could not read nnue_model_weights.json: {e}"))?;
        let j: HeuristicModelJson = serde_json::from_str(&data)
            .map_err(|e| format!("Error deserializing nnue_model_weights.json: {e}"))?;
        Ok(Self {
            w1: j.w1.into_iter().flatten().collect(),
            b1: j.b1,
            w2: j.w2.into_iter().flatten().collect(),
            b2: j.b2,
            w3: j.w3,
            b3: j.b3,
        })
    }

    #[inline]
    pub fn predict(&self, feats: &[usize]) -> f32 {
        let mut h1 = [0.0f32; 256];
        h1.copy_from_slice(&self.b1);
        unsafe {
            let p1 = h1.as_mut_ptr();
            let w = self.w1.as_ptr();
            for &i in feats {
                if i < 480 {
                    sum_row_avx512(p1, w.add(i * 256));
                }
            }
            relu_avx512::<16>(p1);
        }
        let mut h2 = [0.0f32; 128];
        h2.copy_from_slice(&self.b2);
        unsafe {
            let p1 = h1.as_ptr();
            let p2 = h2.as_mut_ptr();
            let w = self.w2.as_ptr();
            for i in 0..256 {
                let v = *p1.add(i);
                if v > 0.0 {
                    fmadd_row_avx512(p2, w.add(i * 128), v);
                }
            }
            relu_avx512::<8>(p2);
        }
        (unsafe { dot128_avx512(h2.as_ptr(), self.w3.as_ptr()) }) + self.b3
    }
}
