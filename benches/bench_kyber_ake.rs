#![cfg(feature = "benchmarking")]
use criterion::{criterion_group, criterion_main, Criterion};
use evo_core_crypto_kyber::*;
use evo_core_id::UId;

// Benchmark keypair generation
fn bench_keypair_generation(c: &mut Criterion) {
    c.bench_function("AKE Keypair Generation", |b| {
        b.iter(|| {
            let mut rng = rand::thread_rng();
            let _keys = keypair(&mut rng);
        })
    });
}

// Benchmark do_client_send
fn bench_do_client_send(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let bob_keys = keypair(&mut rng);
    let id_alice = UId::id();
    
    c.bench_function("AKE do_client_send", |b| {
        b.iter(|| {
            let mut alice = Ake::new();
            let mut rng = rand::thread_rng();
            let _client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
        })
    });
}

// Benchmark on_server_receive
fn bench_on_server_receive(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let _alice_keys = keypair(&mut rng);
    let bob_keys = keypair(&mut rng);
    let id_alice = UId::id();
    
    // Pre-generate client message
    let mut alice = Ake::new();
    let client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
    
    c.bench_function("AKE on_server_receive", |b| {
        b.iter(|| {
            let mut bob = Ake::new();
            let _result = bob.on_server_receive(&client_send, &bob_keys.secret);
        })
    });
}

// Benchmark do_server_send
fn bench_do_server_send(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let alice_keys = keypair(&mut rng);
    let bob_keys = keypair(&mut rng);
    let id_alice = UId::id();
    
    // Pre-generate client message
    let mut alice = Ake::new();
    let client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
    
    c.bench_function("AKE do_server_send", |b| {
        b.iter(|| {
            let mut bob = Ake::new();
            let mut rng = rand::thread_rng();
            let _server_send = bob.do_server_send(&client_send, &alice_keys.public, &bob_keys.secret, &mut rng);
        })
    });
}

// Benchmark do_client_confirm
fn bench_do_client_confirm(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let alice_keys = keypair(&mut rng);
    let bob_keys = keypair(&mut rng);
    let id_alice = UId::id();
    
    // Pre-generate messages
    let mut alice = Ake::new();
    let client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
    
    let mut bob = Ake::new();
    let (_id_client, _temp_key) = bob.on_server_receive(&client_send, &bob_keys.secret).unwrap();
    let server_send = bob.do_server_send(&client_send, &alice_keys.public, &bob_keys.secret, &mut rng).unwrap();
    
    c.bench_function("AKE do_client_confirm", |b| {
        b.iter(|| {
            let mut alice_bench = Ake::new();
            // Need to recreate alice state for each iteration
            let _client_send = alice_bench.do_client_send(&bob_keys.public, id_alice, &mut rand::thread_rng());
            let _result = alice_bench.do_client_confirm(server_send, &alice_keys.secret);
        })
    });
}

// Benchmark full AKE round trip
fn bench_full_ake_roundtrip(c: &mut Criterion) {
    c.bench_function("AKE Full Round Trip", |b| {
        b.iter(|| {
            let mut rng_alice = rand::thread_rng();
            let mut rng_bob = rand::thread_rng();
            
            let mut alice = Ake::new();
            let mut bob = Ake::new();
            
            let alice_keys = keypair(&mut rng_alice);
            let bob_keys = keypair(&mut rng_alice);
            let id_alice = UId::id();
            
            // Full AKE protocol
            let client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng_alice);
            let (_id_client, _temp_key) = bob.on_server_receive(&client_send, &bob_keys.secret).unwrap();
            let server_send = bob.do_server_send(&client_send, &alice_keys.public, &bob_keys.secret, &mut rng_bob).unwrap();
            let _result = alice.do_client_confirm(server_send, &alice_keys.secret);
            
            // Verify shared secrets match
            assert_eq!(alice.shared_secret, bob.shared_secret);
        })
    });
}

criterion_group!(
    ake_benches,
    bench_keypair_generation,
    bench_do_client_send,
    bench_on_server_receive,
    bench_do_server_send,
    bench_do_client_confirm,
    bench_full_ake_roundtrip
);

criterion_main!(ake_benches);