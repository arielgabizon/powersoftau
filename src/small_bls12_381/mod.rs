use blake2b_simd::State as Blake2b;
use byteorder::{BigEndian, ReadBytesExt};
use ff::{Field, PrimeField};
use generic_array::GenericArray;
use paired::bls12_381::Bls12;
use paired::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use std::fmt;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use typenum::consts::U64;

use crate::keypair::*;
use crate::parameters::*;
use crate::utils::*;

#[derive(Clone)]
pub struct Bls12CeremonyParameters {}

impl PowersOfTauParameters for Bls12CeremonyParameters {
    const REQUIRED_POWER: usize = 27; // generate to have roughly 2 million constraints

    // This ceremony is based on the BLS381 elliptic curve construction.
    const G1_UNCOMPRESSED_BYTE_SIZE: usize = 96;
    const G2_UNCOMPRESSED_BYTE_SIZE: usize = 192;
    const G1_COMPRESSED_BYTE_SIZE: usize = 48;
    const G2_COMPRESSED_BYTE_SIZE: usize = 96;
}
