#![allow(clippy::needless_range_loop)]

mod cube;
mod search;

use cube::{Cube, Move, QUARTER_MOVES};
use rand::Rng;
use std::time::Instant;

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

fn generate_scrambled_cube_with_moves(k: usize) -> (Cube, [usize; 20], Vec<Move>) {
    let mut cube = Cube::new();
    let mut rng = rand::thread_rng();
    let mut moves = Vec::new();
    let mut last_move_face = 99;
    for _ in 0..k {
        loop {
            let mv = QUARTER_MOVES[rng.gen_range(0..12)];
            let face = mv.face();
            if face != last_move_face {
                cube.apply_move(mv);
                moves.push(mv);
                last_move_face = face;
                break;
            }
        }
    }
    let features = cube.to_features();
    (cube, features, moves)
}

fn generate_scrambled_cube(k: usize) -> (Cube, [usize; 20]) {
    let (cube, features, _) = generate_scrambled_cube_with_moves(k);
    (cube, features)
}

fn run_benchmark() {
    println!("Running NNUE (AVX-512) vs Dense performance benchmark...");
    let mut rng = rand::thread_rng();
    let hidden_sizes = [512, 1024, 2048, 4096, 8192];
    let mut csv_content = String::from("Hidden_Layer,N,NNUE_Time_ms,Dense_Time_ms,Speedup\n");

    for &m in &hidden_sizes {
        let w1: Vec<Vec<f32>> = (0..480)
            .map(|_| (0..m).map(|_| rng.gen_range(0.0..1.0)).collect())
            .collect();
        let b1: Vec<f32> = (0..m).map(|_| rng.gen_range(0.0..1.0)).collect();
        for n in 3..=12 {
            let mut active_features = Vec::new();
            while active_features.len() < n {
                let idx = rng.gen_range(0..480);
                if !active_features.contains(&idx) {
                    active_features.push(idx);
                }
            }
            let start_nnue = Instant::now();
            let iterations = 10000;
            for _ in 0..iterations {
                let mut h1 = b1.clone();
                let active = std::hint::black_box(&active_features);
                unsafe {
                    for &idx in active {
                        sum_row_avx512(&mut h1, &w1[idx]);
                    }
                    relu_avx512(&mut h1);
                }
                std::hint::black_box(h1);
            }
            let duration_nnue = start_nnue.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

            let mut dense_input = vec![0.0f32; 480];
            for &idx in &active_features {
                dense_input[idx] = 1.0;
            }
            let start_dense = Instant::now();
            for _ in 0..iterations {
                let mut h1 = b1.clone();
                let input = std::hint::black_box(&dense_input);
                for idx in 0..480 {
                    let val = input[idx];
                    let w1_row = &w1[idx];
                    for j in 0..m {
                        h1[j] += val * w1_row[j];
                    }
                }
                for val in &mut h1 {
                    if *val < 0.0 {
                        *val = 0.0;
                    }
                }
                std::hint::black_box(h1);
            }
            let duration_dense = start_dense.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
            let speedup = duration_dense / duration_nnue;
            csv_content.push_str(&format!(
                "{m},{n},{duration_nnue:.6},{duration_dense:.6},{speedup:.2}x\n"
            ));
        }
    }
    std::fs::write("benchmark_results.csv", csv_content)
        .expect("Could not write benchmark_results.csv");
    println!("Benchmark completed. Results written to benchmark_results.csv");
}

fn run_solve_experiment() {
    let m = 256;
    let weights_path = std::path::Path::new("nnue_model_weights.json");
    let _model = if weights_path.exists() {
        let data =
            std::fs::read_to_string(weights_path).expect("Could not read nnue_model_weights.json");
        let loaded_model: NnueRegressor =
            serde_json::from_str(&data).expect("Error deserializing nnue_model_weights.json");
        loaded_model
    } else {
        let mut model = NnueRegressor::new(m);
        let batch_size = 15000;
        let max_epochs = 100;
        let min_epochs = 10;
        let lr = 0.001;
        let mut prev_loss = f32::MAX;
        let mut patience_counter = 0;
        for epoch in 1..=max_epochs {
            let mut total_loss = 0.0;
            for _ in 0..batch_size {
                let mut rng = rand::thread_rng();
                let k = rng.gen_range(1..=20);
                let (_, features) = generate_scrambled_cube(k);
                let (h1, h2, pred) = model.forward(&features);
                let loss = (pred - k as f32).powi(2);
                total_loss += loss;
                model.backward(&features, &h1, &h2, pred, k as f32, lr);
            }
            let avg_loss = total_loss / batch_size as f32;
            if epoch >= min_epochs {
                if avg_loss >= prev_loss || (prev_loss - avg_loss) < 0.005 {
                    patience_counter += 1;
                    if patience_counter >= 3 {
                        break;
                    }
                } else {
                    patience_counter = 0;
                }
            }
            prev_loss = avg_loss;
        }
        let json_str = serde_json::to_string_pretty(&model).expect("Error serializing");
        std::fs::write(weights_path, json_str).expect("Could not write nnue_model_weights.json");
        let mut csv_content =
            String::from("Cube_Num,Real_Steps,Network_Prediction,Absolute_Error\n");
        let mut rng = rand::thread_rng();
        for i in 1..=1000 {
            let k_real = rng.gen_range(1..=20);
            let (_, features) = generate_scrambled_cube(k_real);
            let (_, _, pred) = model.forward(&features);
            let error = (pred - k_real as f32).abs();
            csv_content.push_str(&format!("{i},{k_real},{pred:.4},{error:.4}\n"));
        }
        std::fs::write("nnue_training_eval.csv", csv_content)
            .expect("Could not write nnue_training_eval.csv");
        model
    };

    let solver = search::IdaStarSolver::new();
    let mut csv_results =
        String::from("Cube_Num,K_Scramble,Scramble,Solution,Solution_Length,Time_ms\n");
    let mut cube_num = 1;
    let mut solved_count = 0;
    let mut timeouts = 0;
    let mut solve_times = Vec::new();
    let mut solution_lengths = Vec::new();

    for k in 7..=14 {
        for _ in 0..100 {
            let (mut scrambled_cube, _, scramble_moves) = generate_scrambled_cube_with_moves(k);
            let scramble_strings: Vec<String> = scramble_moves
                .iter()
                .map(|&mov| mov.name().to_string())
                .collect();
            let scramble_joined = scramble_strings.join(" ");
            let start_solve = Instant::now();
            let solution = solver.solve(&scrambled_cube);
            let elapsed_ms = start_solve.elapsed().as_nanos() as f64 / 1_000_000.0;
            let percentage = (cube_num as f32 / 800.0) * 100.0;

            if let Some(sol) = solution {
                for &mov in &sol {
                    scrambled_cube.apply_move(mov);
                }
                assert!(scrambled_cube.is_solved());
                let solution_length = sol.len();
                let solution_strings: Vec<String> =
                    sol.iter().map(|&mov| mov.name().to_string()).collect();
                let solution_joined = solution_strings.join(" ");
                csv_results.push_str(&format!(
                    "{cube_num},{k},\"{scramble_joined}\",\"{solution_joined}\",{solution_length},{elapsed_ms:.4}\n"
                ));
                solved_count += 1;
                solve_times.push(elapsed_ms);
                solution_lengths.push(solution_length);
                println!(
                    "[K={k:<2}] Cube #{cube_num:<4}/800 solved in {elapsed_ms:>8.2} ms | Len: {solution_length:<2} | Progress: {percentage:.1}%"
                );
            } else {
                timeouts += 1;
                csv_results.push_str(&format!(
                    "{cube_num},{k},\"{scramble_joined}\",\"TIMEOUT\",-1,{elapsed_ms:.4}\n"
                ));
                println!(
                    "[K={k:<2}] Cube #{cube_num:<4}/800 TIMEOUT (>10s) | Progress: {percentage:.1}%"
                );
            }
            cube_num += 1;
        }
    }
    std::fs::write("training_evaluation_results.csv", csv_results)
        .expect("Could not write training_evaluation_results.csv");

    let total_attempts = cube_num - 1;
    let success_rate = (solved_count as f32 / total_attempts as f32) * 100.0;
    let avg_time = if solved_count > 0 {
        solve_times.iter().sum::<f64>() / solved_count as f64
    } else {
        0.0
    };
    let avg_len = if solved_count > 0 {
        solution_lengths.iter().sum::<usize>() as f32 / solved_count as f32
    } else {
        0.0
    };
    let min_time = solve_times.iter().copied().fold(f64::INFINITY, f64::min);
    let max_time = solve_times
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    println!("\n=== RESOLUTION STATISTICS ===");
    println!("Total Attempted: {total_attempts}");
    println!("Successfully Solved: {solved_count} / {total_attempts} ({success_rate:.2}%)");
    println!("Timeouts (>60s): {timeouts}");
    if solved_count > 0 {
        println!(
            "Average Solve Time: {avg_time:.2} ms (Min: {min_time:.2} ms, Max: {max_time:.2} ms)"
        );
        println!("Average Solution Length: {avg_len:.2} moves");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--benchmark") {
        run_benchmark();
    } else {
        run_solve_experiment();
    }
}
