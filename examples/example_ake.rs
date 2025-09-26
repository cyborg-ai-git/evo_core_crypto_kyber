//==================================================================================================
use evo_core_crypto_kyber::*;
use evo_core_id::{UId, UIdExt};
use evo_core_log::do_init_logger;
use log::{LevelFilter, debug};
//==================================================================================================
fn main() -> Result<(), KyberError> {
    do_init_logger(Some(LevelFilter::Debug));
    let mut rng_alice = rand::thread_rng();

    let mut alice = Ake::new();
    let mut bob = Ake::new();

    let alice_keys = keypair(&mut rng_alice);
    let bob_keys = keypair(&mut rng_alice);

    //generate from sha256(kyber, dilitium) random as example
    let id_alice = UId::id();

    debug!("id_alice: {}", id_alice.to_hex());

    // Alice initiates key exchange with bob
    let client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng_alice);
    debug!("client_send: {} bytes", client_send.len());

    let temp_sk_alice = alice.temp_key;
    debug!("temp_sk_alice: {}", temp_sk_alice.to_hex());

    // Bob receives the request and authenticates Alice, sends
    // encapsulated shared secret back
    let mut rng_bob = rand::thread_rng();

    let (id_client, server_send, temp_sk_bob) = bob.on_server_receive(
        client_send,
        &alice_keys.public,
        &bob_keys.secret,
        &mut rng_bob,
    )?;

    debug!("id_client: {}", id_client.to_hex());

    assert_eq!(id_alice, id_client);

    let temp_sk_bob_hex = temp_sk_bob
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    debug!("temp_sk_bob: {}", temp_sk_bob_hex);

    debug!("server_send: {} bytes", server_send.len());

    assert_eq!(temp_sk_alice.to_vec(), temp_sk_bob);

    // Alice autheticates and decapsulates
    alice.do_client_confirm(server_send, &alice_keys.secret)?;

    debug!("Alice sk {}", alice.shared_secret.to_hex());
    debug!("Bob sk {}", bob.shared_secret.to_hex());

    // Both structs now have the shared secret
    assert_eq!(alice.shared_secret, bob.shared_secret);

    Ok(())
}
//==================================================================================================
