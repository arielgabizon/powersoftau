extern crate bellman;
extern crate powersoftau;

use paired::bls12_381::Bls12;
use powersoftau::accumulator::Accumulator;
use powersoftau::parameters::UseCompression;
use powersoftau::small_bls12_381::Bls12CeremonyParameters;
use powersoftau::utils::blank_hash;

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

fn main() {
    let writer = OpenOptions::new()
        .read(false)
        .write(true)
        .create_new(true)
        .open("challenge")
        .expect("unable to create `./challenge`");

    let mut writer = BufWriter::new(writer);

    // Write a blank BLAKE2b hash:
    writer
        .write_all(&blank_hash())
        .expect("unable to write blank hash to `./challenge`");

    let parameters = Bls12CeremonyParameters {};

    let acc: Accumulator<Bls12, _> = Accumulator::new(parameters);
    println!("Writing an empty accumulator to disk");
    acc.serialize(&mut writer, UseCompression::No)
        .expect("unable to write fresh accumulator to `./challenge`");
    writer.flush().expect("unable to flush accumulator to disk");

    println!("Wrote a fresh accumulator to `./challenge`");
}
