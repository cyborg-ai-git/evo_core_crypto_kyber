#![cfg(feature = "benchmarking")] // Lint
use criterion::{criterion_group, criterion_main, Criterion};
use evo_core_crypto_kyber::*;


// Benchmarking key generation
fn keypair(c: &mut Criterion) {
  let mut _rng = rand::thread_rng(); //placeholder
  let mut pk = [0u8; KYBER_PUBLICKEYBYTES];
  let mut sk = [0u8; KYBER_SECRETKEYBYTES];
  let bufs = Some(([1u8; 32].as_slice(), [255u8; 32].as_slice()));
  c.bench_function("Keypair Generation", |b| {
    b.iter(|| {
      crypto_kem_keypair(&mut pk, &mut sk, &mut _rng, bufs);
    })
  });
}

// Encapsulating a single public key
fn encap(c: &mut Criterion) {
  let mut ct = [0u8; KYBER_CIPHERTEXTBYTES];
  let mut ss = [0u8; KYBER_SSBYTES];
  // Generate fresh keys for the current Kyber variant
  let mut pk = [0u8; KYBER_PUBLICKEYBYTES];
  let mut sk = [0u8; KYBER_SECRETKEYBYTES];
  let mut rng = rand::thread_rng();
  crypto_kem_keypair(&mut pk, &mut sk, &mut rng, None);
  
  let encap_buf = Some([255u8; 32].as_slice());
  c.bench_function("Encapsulate", |b| {
    b.iter(|| {
      crypto_kem_enc(&mut ct, &mut ss, &pk, &mut rng, encap_buf);
    })
  });
}

// Decapsulating a single correct ciphertext
fn decap(c: &mut Criterion) {
  // Generate fresh keys and ciphertext for the current Kyber variant
  let mut pk = [0u8; KYBER_PUBLICKEYBYTES];
  let mut sk = [0u8; KYBER_SECRETKEYBYTES];
  let mut ct = [0u8; KYBER_CIPHERTEXTBYTES];
  let mut ss = [0u8; KYBER_SSBYTES];
  let mut rng = rand::thread_rng();
  
  crypto_kem_keypair(&mut pk, &mut sk, &mut rng, None);
  crypto_kem_enc(&mut ct, &mut ss, &pk, &mut rng, None);
  
  c.bench_function("Decapsulate", |b| {
    b.iter(|| {
      let _dec = decapsulate(&ct, &sk);
    })
  });
}

// Decapsulating a single incorrect ciphertext
fn decap_fail(c: &mut Criterion) {
  // Generate fresh keys and ciphertext, then corrupt the ciphertext
  let mut pk = [0u8; KYBER_PUBLICKEYBYTES];
  let mut sk = [0u8; KYBER_SECRETKEYBYTES];
  let mut ct = [0u8; KYBER_CIPHERTEXTBYTES];
  let mut ss = [0u8; KYBER_SSBYTES];
  let mut rng = rand::thread_rng();
  
  crypto_kem_keypair(&mut pk, &mut sk, &mut rng, None);
  crypto_kem_enc(&mut ct, &mut ss, &pk, &mut rng, None);
  
  // Corrupt the first few bytes of the ciphertext to simulate failure
  ct[0] = ct[0].wrapping_add(1);
  ct[1] = ct[1].wrapping_add(1);
  
  c.bench_function("Decapsulate Failure", |b| {
    b.iter(|| {
      let _dec = decapsulate(&ct, &sk);
    })
  });
}

criterion_group!(benches, keypair, encap, decap, decap_fail);
criterion_main!(benches);
