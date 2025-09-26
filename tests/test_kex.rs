use evo_core_crypto_kyber::*;
use evo_core_id::UId;

// Kyber struct uake and ake functions
#[test]
fn uake_valid() {
  let mut rng = rand::thread_rng();
  let mut alice = Uake::new();
  let mut bob = Uake::new();
  let bob_keys = keypair(&mut rng);
  let client_init = alice.client_init(&bob_keys.public, &mut rng);
  let server_send = bob.server_receive(client_init, &bob_keys.secret, &mut rng).unwrap();
  alice.client_confirm(server_send).unwrap();
  assert_eq!(alice.shared_secret, bob.shared_secret);
}

// Corrupted ciphertext sent to bob, 4 bytes modified
#[test]
fn uake_invalid_client_init_ciphertext() {
  let mut rng = rand::thread_rng();
  let mut alice = Uake::new();
  let mut bob = Uake::new();
  let bob_keys = keypair(&mut rng);
  let mut client_init = alice.client_init(&bob_keys.public, &mut rng);
  client_init[KYBER_PUBLICKEYBYTES..][..4].copy_from_slice(&[255u8;4]);
  assert!(!bob.server_receive(client_init, &bob_keys.secret, &mut rng).is_err());
  assert_ne!(alice.shared_secret, bob.shared_secret);
}

// Corrupted public key sent to bob, detected by Alice
#[test]
fn uake_invalid_client_init_publickey() {
  let mut rng = rand::thread_rng();
  let mut alice = Uake::new();
  let mut bob = Uake::new();
  let bob_keys = keypair(&mut rng);
  let mut client_init = alice.client_init(&bob_keys.public, &mut rng);
  client_init[..4].copy_from_slice(&[255u8;4]);
  let server_send = bob.server_receive(client_init, &bob_keys.secret, &mut rng).unwrap();
  assert!(!alice.client_confirm(server_send).is_err());
  assert_ne!(alice.shared_secret, bob.shared_secret);
}

// Corrupted ciphertext sent back to Alice
#[test]
fn uake_invalid_server_send_ciphertext() {
  let mut rng = rand::thread_rng();
  let mut alice = Uake::new();
  let mut bob = Uake::new();
  let bob_keys = keypair(&mut rng);
  let client_init = alice.client_init(&bob_keys.public, &mut rng);
  let mut server_send = bob.server_receive(client_init, &bob_keys.secret, &mut rng).unwrap();
  server_send[..4].copy_from_slice(&[255u8;4]);
  assert!(!alice.client_confirm(server_send).is_err());
  assert_ne!(alice.shared_secret, bob.shared_secret);
}

// Same tests for AKE

#[test]
fn ake_valid() {
  let mut rng = rand::thread_rng();
  let mut alice = Ake::new();
  let mut bob = Ake::new();
  let alice_keys = keypair(&mut rng);
  let bob_keys = keypair(&mut rng);
  let id_alice = UId::id_hex("9fbbbd21c05a39e52726048832c5af044f559c4723523dd869a481bb8781b525");
  let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
  let (_client_id, server_send, _temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng).unwrap();
  let _client_confirm = alice.do_client_confirm(server_send, &alice_keys.secret).unwrap();
  assert_eq!(alice.shared_secret, bob.shared_secret);
}

#[test]
fn ake_invalid_client_init_ciphertext() {
  let mut rng = rand::thread_rng();
  let mut alice = Ake::new();
  let mut bob = Ake::new();
  let alice_keys = keypair(&mut rng);
  let bob_keys = keypair(&mut rng);
  let id_alice = UId::id_hex("9fbbbd21c05a39e52726048832c5af044f559c4723523dd869a481bb8781b525");
  let mut client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
  client_init[KYBER_PUBLICKEYBYTES..][..4].copy_from_slice(&[255u8;4]);
  assert!(!bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng).is_err());
  assert_ne!(alice.shared_secret, bob.shared_secret);
}

#[test]
fn ake_invalid_client_init_publickey() {
  let mut rng = rand::thread_rng();
  let mut alice = Ake::new();
  let mut bob = Ake::new();
  let alice_keys = keypair(&mut rng);
  let bob_keys = keypair(&mut rng);
  let id_alice = UId::id_hex("9fbbbd21c05a39e52726048832c5af044f559c4723523dd869a481bb8781b525");
  let mut client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
  client_init[..4].copy_from_slice(&[255u8;4]);
  let (_client_id, server_send, _temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng).unwrap();
  assert!(!alice.do_client_confirm(server_send, &alice_keys.secret).is_err());
  assert_ne!(alice.shared_secret, bob.shared_secret);
}

#[test]
fn ake_invalid_server_send_first_ciphertext() {
  let mut rng = rand::thread_rng();
  let mut alice = Ake::new();
  let mut bob = Ake::new();
  let alice_keys = keypair(&mut rng);
  let bob_keys = keypair(&mut rng);
  let id_alice = UId::id_hex("9fbbbd21c05a39e52726048832c5af044f559c4723523dd869a481bb8781b525");
  let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
  let (_client_id, mut server_send, _temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng).unwrap();
  server_send[..4].copy_from_slice(&[255u8;4]);
  assert!(!alice.do_client_confirm(server_send, &alice_keys.secret).is_err());
  assert_ne!(alice.shared_secret, bob.shared_secret);
}

#[test]
fn ake_invalid_server_send_second_ciphertext() {
  let mut rng = rand::thread_rng();
  let mut alice = Ake::new();
  let mut bob = Ake::new();
  let alice_keys = keypair(&mut rng);
  let bob_keys = keypair(&mut rng);
  let id_alice = UId::id_hex("9fbbbd21c05a39e52726048832c5af044f559c4723523dd869a481bb8781b525");
  let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
  let (_client_id, mut server_send, _temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng).unwrap();
  server_send[KYBER_CIPHERTEXTBYTES..][..4].copy_from_slice(&[255u8;4]);
  // assert!(alice.do_client_confirm(server_send, &alice_keys.secret).is_err());
}
