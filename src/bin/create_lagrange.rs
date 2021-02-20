use std::fs::OpenOptions;
use std::io::{self, BufReader, BufWriter, Read, Write};

use bellman::domain::{EvaluationDomain, Point};
use bellman::gpu;
use bellman::multicore::Worker;
use groupy::{CurveAffine, CurveProjective};
use log::info;
use paired::bls12_381::{Bls12, G1, G2};
use powersoftau::accumulator::*;
use powersoftau::keypair::PublicKey;
use powersoftau::parameters::{CheckForCorrectness, PowersOfTauParameters, UseCompression};
use powersoftau::small_bls12_381::Bls12CeremonyParameters;
use rayon::prelude::*;

fn into_hex(h: &[u8]) -> String {
    let mut f = String::new();

    for byte in &h[..] {
        f += &format!("{:02x}", byte);
    }

    f
}

// Computes the hash of the challenge file for the player,
// given the current state of the accumulator and the last
// response file hash.
fn get_challenge_file_hash(
    acc: &Accumulator<Bls12, Bls12CeremonyParameters>,
    last_response_file_hash: &[u8; 64],
) -> [u8; 64] {
    let sink = io::sink();
    let mut sink = HashWriter::new(sink);

    sink.write_all(last_response_file_hash).unwrap();

    acc.serialize(&mut sink, UseCompression::No).unwrap();

    sink.into_hash()
}

// Computes the hash of the response file, given the new
// accumulator, the player's public key, and the challenge
// file's hash.
fn get_response_file_hash(
    acc: &Accumulator<Bls12, Bls12CeremonyParameters>,
    pubkey: &PublicKey<Bls12>,
    last_challenge_file_hash: &[u8; 64],
) -> [u8; 64] {
    let sink = io::sink();
    let mut sink = HashWriter::new(sink);

    sink.write_all(last_challenge_file_hash).unwrap();

    acc.serialize(&mut sink, UseCompression::Yes).unwrap();

    pubkey.serialize(&mut sink).unwrap();

    sink.into_hash()
}

fn main() {
    pretty_env_logger::init_timed();

    // Try to load `./challenge` from disk.
    info!("Loading challenge");
    let challenge_reader = OpenOptions::new()
        .read(true)
        .open("challenge")
        .expect("unable open `./challenge` in this directory");

    {
        let metadata = challenge_reader
            .metadata()
            .expect("unable to get filesystem metadata for `./challenge`");
        if metadata.len() != (Bls12CeremonyParameters::ACCUMULATOR_BYTE_SIZE as u64) {
            panic!(
                "The size of `./challenge` should be {}, but it's {}, so something isn't right.",
                Bls12CeremonyParameters::ACCUMULATOR_BYTE_SIZE,
                metadata.len()
            );
        }
    }

    let challenge_reader = BufReader::new(challenge_reader);
    let mut challenge_reader = HashReader::new(challenge_reader);

    // Deserialize the current challenge

    // Read the BLAKE2b hash of the previous contribution
    {
        // We don't need to do anything with it, but it's important for
        // the hash chain.
        let mut tmp = [0; 64];
        challenge_reader
            .read_exact(&mut tmp)
            .expect("unable to read BLAKE2b hash of previous contribution");
    }
    let parameters = Bls12CeremonyParameters {};

    // Load the current accumulator into memory
    let current_accumulator = Accumulator::<Bls12, Bls12CeremonyParameters>::deserialize(
        &mut challenge_reader,
        UseCompression::No,
        CheckForCorrectness::No, // no need to check since we constructed the challenge already
        parameters,
    )
    .expect("unable to read uncompressed accumulator");

    //    // Try to load `./response` from disk.
    //    let response_reader = OpenOptions::new()
    //        .read(true)
    //        .open("response").expect("unable open `./response` in this directory");
    //
    //    {
    //        let metadata = response_reader.metadata().expect("unable to get filesystem metadata for `./response`");
    //        if metadata.len() != (CONTRIBUTION_BYTE_SIZE as u64) {
    //            panic!("The size of `./response` should be {}, but it's {}, so something isn't right.", CONTRIBUTION_BYTE_SIZE, metadata.len());
    //        }
    //    }
    //
    //    let response_reader = BufReader::new(response_reader);
    //    let mut response_reader = HashReader::new(response_reader);
    //    // Read hash (just to move reader to right place in file to begin deserialization of accumulator)
    //    {
    //        let mut response_challenge_hash = [0; 64];
    //        response_reader.read_exact(&mut response_challenge_hash).expect("couldn't read hash of challenge file from response file");
    ////
    ////        if &response_challenge_hash[..] != current_accumulator_hash.as_slice() {
    ////            panic!("Hash chain failure. This is not the right response.");
    ////        }
    //    }
    //// Load the response's accumulator
    //
    //    let current_accumulator = Accumulator::deserialize(&mut response_reader, UseCompression::Yes, CheckForCorrectness::Yes)
    //        .expect("wasn't able to deserialize the response file's accumulator");
    //    // Initialize the accumulator
    //    let mut current_accumulator = Accumulator::new();

    //    // The "last response file hash" is just a blank BLAKE2b hash
    //    // at the beginning of the hash chain.
    //    let mut last_response_file_hash = [0; 64];
    //    last_response_file_hash.copy_from_slice(blank_hash().as_slice());
    //
    //    // There were 89 rounds.
    //    //  for _ in 0..89 {
    //    // Compute the hash of the challenge file that the player
    //    // should have received.
    //    let last_challenge_file_hash = get_challenge_file_hash(
    //        &current_accumulator,
    //        &last_response_file_hash
    //    );
    //
    //    // Deserialize the accumulator provided by the player in
    //    // their response file. It's stored in the new_challenge in
    //    // uncompressed form so that we can more efficiently
    //    // deserialize it.
    //    let response_file_accumulator = Accumulator::deserialize(
    //        &mut response_readable_map,
    //        UseCompression::Yes,
    //        CheckForCorrectness::No
    //    ).expect("unable to read uncompr  essed accumulator");

    // Deserialize the public key provided by the player.
    //        let response_file_pubkey = PublicKey::deserialize(&mut reader)
    //            .expect("wasn't able to deserialize the response file's public key");

    // Compute the hash of the response file. (we had it in uncompressed
    // form in the transcript, but the response file is compressed to save
    // participants bandwidth.)
    //        last_response_file_hash = get_response_file_hash(
    //            &response_file_accumulator,
    //            &response_file_pubkey,
    //            &last_challenge_file_hash
    //        );

    //    print!("{}", into_hex(&last_response_file_hash));

    // Verify the transformation from the previous accumulator to the new
    // one. This also verifies the correctness of the accumulators and the
    // public keys, with respect to the transcript so far.
    //        if !verify_transform(
    //            &current_accumulator,
    //            &response_file_accumulator,
    //            &response_file_pubkey,
    //            &last_challenge_file_hash
    //        )
    //        {
    //            info!(" ... FAILED");
    //            panic!("INVALID RESPONSE FILE!");
    //        } else {
    //            info!("");
    //        }

    //    current_accumulator = response_file_accumulator;
    //}

    info!("Transcript OK!");

    let worker = &Worker::new();
    let lock = gpu::lock().expect("failed to aquire gpu lock");

    // Create the parameters for various 2^m circuit depths.
    for m in 24..28 {
        let paramname = format!("phase1radix2m{}", m);
        info!("\n\nCreating {}", paramname);

        let degree = 1 << m;
        let mut fft_kern = bellman::domain::gpu_fft_supported::<Bls12>(m).ok();

        if fft_kern.is_some() {
            info!("GPU FFT is supported!");
        } else {
            info!("GPU FFT is NOT supported!");
        }

        info!("Creating g1_coeffs");

        let mut g1_coeffs = EvaluationDomain::from_coeffs(
            current_accumulator.tau_powers_g1[0..degree]
                .iter()
                .map(|e| Point(e.into_projective()))
                .collect(),
        )
        .unwrap();
        info!("Creating g2_coeffs");

        let mut g2_coeffs = EvaluationDomain::from_coeffs(
            current_accumulator.tau_powers_g2[0..degree]
                .iter()
                .map(|e| Point(e.into_projective()))
                .collect(),
        )
        .unwrap();

        info!("Creating g1_alpha_coeffs");
        let mut g1_alpha_coeffs = EvaluationDomain::from_coeffs(
            current_accumulator.alpha_tau_powers_g1[0..degree]
                .iter()
                .map(|e| Point(e.into_projective()))
                .collect(),
        )
        .unwrap();

        info!("Creating g1_beta_coeffs");
        let mut g1_beta_coeffs = EvaluationDomain::from_coeffs(
            current_accumulator.beta_tau_powers_g1[0..degree]
                .iter()
                .map(|e| Point(e.into_projective()))
                .collect(),
        )
        .unwrap();

        // This converts all of the elements into Lagrange coefficients
        // for later construction of interpolation polynomials

        info!("Creating g1_coeffs_ifft");
        g1_coeffs.ifft(&worker, &mut fft_kern).unwrap();
        info!("Creating g2_coeffs_ifft");
        g2_coeffs.ifft(&worker, &mut fft_kern).unwrap();
        info!("Creating g1_alpha_coeffs_ifft");
        g1_alpha_coeffs.ifft(&worker, &mut fft_kern).unwrap();
        info!("Creating g1_beta_coeffs_ifft");
        g1_beta_coeffs.ifft(&worker, &mut fft_kern).unwrap();

        let g1_coeffs = g1_coeffs.into_coeffs();
        let g2_coeffs = g2_coeffs.into_coeffs();
        let g1_alpha_coeffs = g1_alpha_coeffs.into_coeffs();
        let g1_beta_coeffs = g1_beta_coeffs.into_coeffs();

        assert_eq!(g1_coeffs.len(), degree);
        assert_eq!(g2_coeffs.len(), degree);
        assert_eq!(g1_alpha_coeffs.len(), degree);
        assert_eq!(g1_beta_coeffs.len(), degree);

        // Remove the Point() wrappers
        let mut g1_coeffs = g1_coeffs.into_iter().map(|e| e.0).collect::<Vec<_>>();
        let mut g2_coeffs = g2_coeffs.into_iter().map(|e| e.0).collect::<Vec<_>>();
        let mut g1_alpha_coeffs = g1_alpha_coeffs.into_iter().map(|e| e.0).collect::<Vec<_>>();
        let mut g1_beta_coeffs = g1_beta_coeffs.into_iter().map(|e| e.0).collect::<Vec<_>>();

        // Batch normalize
        info!("Batch normalize");
        G1::batch_normalization(&mut g1_coeffs);
        G2::batch_normalization(&mut g2_coeffs);
        G1::batch_normalization(&mut g1_alpha_coeffs);
        G1::batch_normalization(&mut g1_beta_coeffs);

        // H query of Groth16 needs...
        // x^i * (x^m - 1) for i in 0..=(m-2) a.k.a.
        // x^(i + m) - x^i for i in 0..=(m-2)
        // for radix2 evaluation domains
        info!("H query");
        let mut h: Vec<_> = (0..degree - 1)
            .into_par_iter()
            .map(|i| {
                let mut tmp = current_accumulator.tau_powers_g1[i + degree].into_projective();
                let mut tmp2 = current_accumulator.tau_powers_g1[i].into_projective();
                tmp2.negate();
                tmp.add_assign(&tmp2);
                tmp
            })
            .collect();

        info!("Batch normalize H");
        // Batch normalize this as well
        G1::batch_normalization(&mut h);

        info!("Writing results to disk");
        // Create the parameter file
        let writer = OpenOptions::new()
            .read(false)
            .write(true)
            .create_new(true)
            .open(paramname)
            .expect("unable to create parameter file in this directory");

        let mut writer = BufWriter::new(writer);

        // Write alpha (in g1)
        // Needed by verifier for e(alpha, beta)
        // Needed by prover for A and C elements of proof
        writer
            .write_all(
                current_accumulator.alpha_tau_powers_g1[0]
                    .into_uncompressed()
                    .as_ref(),
            )
            .unwrap();

        // Write beta (in g1)
        // Needed by prover for C element of proof
        writer
            .write_all(
                current_accumulator.beta_tau_powers_g1[0]
                    .into_uncompressed()
                    .as_ref(),
            )
            .unwrap();

        // Write beta (in g2)
        // Needed by verifier for e(alpha, beta)
        // Needed by prover for B element of proof
        writer
            .write_all(current_accumulator.beta_g2.into_uncompressed().as_ref())
            .unwrap();

        // Lagrange coefficients in G1 (for constructing
        // LC/IC queries and precomputing polynomials for A)
        let g1_coeffs_uncompressed: Vec<_> = g1_coeffs
            .into_par_iter()
            .map(|c| {
                // Was normalized earlier in parallel
                c.into_affine().into_uncompressed()
            })
            .collect();

        for coeff in g1_coeffs_uncompressed {
            writer.write_all(coeff.as_ref()).unwrap();
        }

        // Lagrange coefficients in G2 (for precomputing
        // polynomials for B)
        let g2_coeffs_uncompressed: Vec<_> = g2_coeffs
            .into_par_iter()
            .map(|c| {
                // Was normalized earlier in parallel
                c.into_affine().into_uncompressed()
            })
            .collect();

        for coeff in g2_coeffs_uncompressed {
            writer.write_all(coeff.as_ref()).unwrap();
        }

        // Lagrange coefficients in G1 with alpha (for
        // LC/IC queries)
        let g1_alpha_coeffs_uncompressed: Vec<_> = g1_alpha_coeffs
            .into_par_iter()
            .map(|c| {
                // Was normalized earlier in parallel
                c.into_affine().into_uncompressed()
            })
            .collect();

        for coeff in g1_alpha_coeffs_uncompressed {
            writer.write_all(coeff.as_ref()).unwrap();
        }

        // Lagrange coefficients in G1 with beta (for
        // LC/IC queries)
        let g1_beta_coeffs_uncompressed: Vec<_> = g1_beta_coeffs
            .into_par_iter()
            .map(|c| {
                // Was normalized earlier in parallel
                c.into_affine().into_uncompressed()
            })
            .collect();

        for coeff in g1_beta_coeffs_uncompressed {
            writer.write_all(coeff.as_ref()).unwrap();
        }

        // Bases for H polynomial computation
        let h_uncompressed: Vec<_> = h
            .into_par_iter()
            .map(|c| {
                // Was normalized earlier in parallel
                c.into_affine().into_uncompressed()
            })
            .collect();

        for coeff in h_uncompressed {
            writer.write_all(coeff.as_ref()).unwrap();
        }
    }

    gpu::unlock(lock);
}
