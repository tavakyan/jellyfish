// Copyright (c) 2022 TRI (spectrum.xyz)
// This file is part of the Jellyfish library.

// You should have received a copy of the MIT License
// along with the Jellyfish library. If not, see <https://mit-license.org/>.

// For benchmark, run:
//     RAYON_NUM_THREADS=N cargo bench --features bench
// where N is the number of threads you want to use (N = 1 for single-thread).

use ark_bls12_377::{Bls12_377, Fr as Fr377};
use ark_bls12_381::{Bls12_381, Fr as Fr381};
use ark_bn254::{Bn254, Fr as Fr254};
use ark_bw6_761::{Fr as Fr761, BW6_761};
use ark_ff::PrimeField;
use ark_std::{fs::File, io::Write};
use jf_plonk::{
    bencher::{init_timers, total_fft_time, total_msm_time, total_poly_eval_time},
    circuit::{Circuit, PlonkCircuit},
    errors::PlonkError,
    proof_system::{PlonkKzgSnark, Snark},
    transcript::StandardTranscript,
    PlonkType,
};

const NUM_REPETITIONS: usize = 10;
const NUM_GATES_LARGE: usize = 32768;
const NUM_GATES_SMALL: usize = 8192;

fn gen_circuit_for_bench<F: PrimeField>(
    num_gates: usize,
    plonk_type: PlonkType,
) -> Result<PlonkCircuit<F>, PlonkError> {
    let range_bit_len = 8;
    let mut cs: PlonkCircuit<F> = match plonk_type {
        PlonkType::TurboPlonk => PlonkCircuit::new_turbo_plonk(),
        PlonkType::UltraPlonk => PlonkCircuit::new_ultra_plonk(range_bit_len),
    };
    let mut a = cs.zero();
    for _ in 0..num_gates - 10 {
        a = cs.add(a, cs.one())?;
    }
    // Finalize the circuit.
    cs.finalize_for_arithmetization()?;

    Ok(cs)
}

macro_rules! plonk_prove_bench {
    ($bench_curve:ty, $bench_field:ty, $bench_plonk_type:expr, $num_gates:expr) => {
        let rng = &mut ark_std::test_rng();
        let cs = gen_circuit_for_bench::<$bench_field>($num_gates, $bench_plonk_type).unwrap();

        let max_degree = $num_gates + 2;
        let srs = PlonkKzgSnark::<$bench_curve>::universal_setup(max_degree, rng).unwrap();

        let (pk, _) = PlonkKzgSnark::<$bench_curve>::preprocess(&srs, &cs).unwrap();

        init_timers();
        let start = ark_std::time::Instant::now();

        for _ in 0..NUM_REPETITIONS {
            let _ = PlonkKzgSnark::<$bench_curve>::prove::<_, _, StandardTranscript>(
                rng, &cs, &pk, None,
            )
            .unwrap();
        }
        println!("=====================================");
        println!(
            "proving time for {}, {} with dim {}: {} ns/gate",
            stringify!($bench_curve),
            stringify!($bench_plonk_type),
            $num_gates,
            start.elapsed().as_nanos() / NUM_REPETITIONS as u128 / $num_gates as u128
        );
        println!(
            "total batch verify time: {:.2} ms",
            start.elapsed().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64
        );
        println!(
            "time spend on FFT:  {:.2} ms, or {:.2}%",
            total_fft_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_fft_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!(
            "time spend on MSM:  {:.2} ms, or {:.2}%",
            total_msm_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_msm_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!(
            "time spend on poly evaluation: {:.2} ms, or {:.2}%",
            total_poly_eval_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_poly_eval_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!("=====================================");
    };
}

macro_rules! plonk_prove_mt_bench {
    ($bench_curve:ty, $bench_field:ty, $bench_plonk_type:expr, $num_gates:expr, $file:expr) => {
        let rng = &mut ark_std::test_rng();
        let cs = gen_circuit_for_bench::<$bench_field>($num_gates, $bench_plonk_type).unwrap();

        let max_degree = $num_gates + 2;
        let srs = PlonkKzgSnark::<$bench_curve>::universal_setup(max_degree, rng).unwrap();

        let (pk, _) = PlonkKzgSnark::<$bench_curve>::preprocess(&srs, &cs).unwrap();

        init_timers();
        let start = ark_std::time::Instant::now();

        for _ in 0..NUM_REPETITIONS {
            let _ = PlonkKzgSnark::<$bench_curve>::prove::<_, _, StandardTranscript>(
                rng, &cs, &pk, None,
            )
            .unwrap();
        }
        println!("=====================================");
        println!(
            "proving time for {}, {} with dim {}: {} ns/gate",
            stringify!($bench_curve),
            stringify!($bench_plonk_type),
            $num_gates,
            start.elapsed().as_nanos() / NUM_REPETITIONS as u128 / $num_gates as u128
        );
        println!(
            "total batch verify time: {:.2} ms",
            start.elapsed().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64
        );
        println!(
            "time spend on FFT:  {:.2} ms, or {:.2}%",
            total_fft_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_fft_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!(
            "time spend on MSM:  {:.2} ms, or {:.2}%",
            total_msm_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_msm_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!(
            "time spend on poly evaluation: {:.2} ms, or {:.2}%",
            total_poly_eval_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_poly_eval_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!("=====================================");
        $file
            .write_all(
                format!(
                    "{}     {:.2}   {:.2}   {:.2}   {:.2}   {:.2}\n",
                    $num_gates,
                    start.elapsed().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
                    total_fft_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
                    100f64 * total_fft_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64,
                    total_msm_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
                    100f64 * total_msm_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64,
                )
                .as_ref(),
            )
            .expect("Unable to write data");
    };
}

fn bench_prove() {
    plonk_prove_bench!(Bls12_381, Fr381, PlonkType::TurboPlonk, NUM_GATES_LARGE);
    plonk_prove_bench!(Bls12_377, Fr377, PlonkType::TurboPlonk, NUM_GATES_LARGE);
    plonk_prove_bench!(Bn254, Fr254, PlonkType::TurboPlonk, NUM_GATES_LARGE);
    plonk_prove_bench!(BW6_761, Fr761, PlonkType::TurboPlonk, NUM_GATES_SMALL);
    plonk_prove_bench!(Bls12_381, Fr381, PlonkType::UltraPlonk, NUM_GATES_LARGE);
    plonk_prove_bench!(Bls12_377, Fr377, PlonkType::UltraPlonk, NUM_GATES_LARGE);
    plonk_prove_bench!(Bn254, Fr254, PlonkType::UltraPlonk, NUM_GATES_LARGE);
    plonk_prove_bench!(BW6_761, Fr761, PlonkType::UltraPlonk, NUM_GATES_SMALL);
}

macro_rules! plonk_verify_bench {
    ($bench_curve:ty, $bench_field:ty, $bench_plonk_type:expr, $num_gates:expr) => {
        let rng = &mut ark_std::test_rng();
        let cs = gen_circuit_for_bench::<$bench_field>($num_gates, $bench_plonk_type).unwrap();

        let max_degree = $num_gates + 2;
        let srs = PlonkKzgSnark::<$bench_curve>::universal_setup(max_degree, rng).unwrap();

        let (pk, vk) = PlonkKzgSnark::<$bench_curve>::preprocess(&srs, &cs).unwrap();

        let proof =
            PlonkKzgSnark::<$bench_curve>::prove::<_, _, StandardTranscript>(rng, &cs, &pk, None)
                .unwrap();

        init_timers();
        let start = ark_std::time::Instant::now();

        for _ in 0..NUM_REPETITIONS {
            let _ =
                PlonkKzgSnark::<$bench_curve>::verify::<StandardTranscript>(&vk, &[], &proof, None)
                    .unwrap();
        }
        println!("=====================================");
        println!(
            "verifying time for {}, {} with dim {}: {} ns",
            stringify!($bench_curve),
            stringify!($bench_plonk_type),
            $num_gates,
            start.elapsed().as_nanos() / NUM_REPETITIONS as u128
        );
        println!(
            "total batch verify time: {:.2} ms",
            start.elapsed().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64
        );
        println!(
            "time spend on FFT:  {:.2} ms, or {:.2}%",
            total_fft_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_fft_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!(
            "time spend on MSM:  {:.2} ms, or {:.2}%",
            total_msm_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_msm_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );

        println!("=====================================");
    };
}

fn bench_verify() {
    plonk_verify_bench!(Bls12_381, Fr381, PlonkType::TurboPlonk, NUM_GATES_LARGE);
    plonk_verify_bench!(Bls12_377, Fr377, PlonkType::TurboPlonk, NUM_GATES_LARGE);
    plonk_verify_bench!(Bn254, Fr254, PlonkType::TurboPlonk, NUM_GATES_LARGE);
    plonk_verify_bench!(BW6_761, Fr761, PlonkType::TurboPlonk, NUM_GATES_SMALL);
    plonk_verify_bench!(Bls12_381, Fr381, PlonkType::UltraPlonk, NUM_GATES_LARGE);
    plonk_verify_bench!(Bls12_377, Fr377, PlonkType::UltraPlonk, NUM_GATES_LARGE);
    plonk_verify_bench!(Bn254, Fr254, PlonkType::UltraPlonk, NUM_GATES_LARGE);
    plonk_verify_bench!(BW6_761, Fr761, PlonkType::UltraPlonk, NUM_GATES_SMALL);
}

macro_rules! plonk_batch_verify_bench {
    ($bench_curve:ty, $bench_field:ty, $bench_plonk_type:expr, $num_proofs:expr) => {
        let rng = &mut ark_std::test_rng();
        let cs = gen_circuit_for_bench::<$bench_field>(1024, $bench_plonk_type).unwrap();

        let max_degree = 1026;
        let srs = PlonkKzgSnark::<$bench_curve>::universal_setup(max_degree, rng).unwrap();

        let (pk, vk) = PlonkKzgSnark::<$bench_curve>::preprocess(&srs, &cs).unwrap();

        let proof =
            PlonkKzgSnark::<$bench_curve>::prove::<_, _, StandardTranscript>(rng, &cs, &pk, None)
                .unwrap();

        let vks = vec![&vk; $num_proofs];
        let pub_input = vec![];
        let public_inputs_ref = vec![&pub_input[..]; $num_proofs];
        let proofs_ref = vec![&proof; $num_proofs];

        init_timers();
        let start = ark_std::time::Instant::now();

        for _ in 0..NUM_REPETITIONS {
            let _ = PlonkKzgSnark::<$bench_curve>::batch_verify::<StandardTranscript>(
                &vks,
                &public_inputs_ref[..],
                &proofs_ref,
                &vec![None; vks.len()],
            )
            .unwrap();
        }

        println!(
            "batch verifying time for {}, {}, {} proofs: {} ns/proof",
            stringify!($bench_curve),
            stringify!($bench_plonk_type),
            stringify!($num_proofs),
            start.elapsed().as_nanos() / NUM_REPETITIONS as u128 / $num_proofs as u128
        );

        println!(
            "total batch verify time: {:.2} ms",
            start.elapsed().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64
        );
        println!(
            "time spend on FFT:  {:.2} ms, or {:.2}%",
            total_fft_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_fft_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
        println!(
            "time spend on MSM:  {:.2} ms, or {:.2}%",
            total_msm_time().as_nanos() as f64 / NUM_REPETITIONS as f64 / 1_000_000f64,
            100f64 * total_msm_time().as_nanos() as f64 / start.elapsed().as_nanos() as f64
        );
    };
}

fn bench_batch_verify() {
    plonk_batch_verify_bench!(Bls12_381, Fr381, PlonkType::TurboPlonk, 1000);
    plonk_batch_verify_bench!(Bls12_377, Fr377, PlonkType::TurboPlonk, 1000);
    plonk_batch_verify_bench!(Bn254, Fr254, PlonkType::TurboPlonk, 1000);
    plonk_batch_verify_bench!(BW6_761, Fr761, PlonkType::TurboPlonk, 1000);
    plonk_batch_verify_bench!(Bls12_381, Fr381, PlonkType::UltraPlonk, 1000);
    plonk_batch_verify_bench!(Bls12_377, Fr377, PlonkType::UltraPlonk, 1000);
    plonk_batch_verify_bench!(Bn254, Fr254, PlonkType::UltraPlonk, 1000);
    plonk_batch_verify_bench!(BW6_761, Fr761, PlonkType::UltraPlonk, 1000);
}

fn bench_intense() {
    let mut f = File::create(format!(
        "../target/{}-threads.txt",
        rayon::current_num_threads()
    ))
    .expect("Unable to create file");

    for i in 10..=30 {
        let dim = 1 << i;
        println!("bench with log(dim) =  {}", i);
        plonk_prove_mt_bench!(Bls12_377, Fr377, PlonkType::TurboPlonk, dim, f);
    }

    for i in 10..=30 {
        let dim = 1 << i;
        println!("bench with log(dim) =  {}", i);
        plonk_verify_bench!(Bls12_377, Fr377, PlonkType::TurboPlonk, dim);
    }
}

fn main() {
    bench_intense();
    // bench_prove();
    // bench_verify();
    // bench_batch_verify();
}
