//! # Kyber
//! 
//! A rust implementation of the Kyber algorithm
//! 
//! This library:
//! * Is no_std compatible and uses no allocations, suitable for embedded devices. 
//! * The reference files contain no unsafe code.
//! * Compiles to WASM using wasm-bindgen.
//! 
//! ## Features
//! If no security level is set then kyber768 is used, this is roughly equivalent to AES-196. See below for setting other levels.
//! A compile-time error is raised if more than one level is specified. Besides that all other features can be mixed as needed:
//!
//! | Feature   | Description                                                                                                                                                                |
//! |-----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
//! | kyber512  | Enables kyber512 mode, with a security level roughly equivalent to AES-128.                                                                                                |
//! | kyber1024 | Enables kyber1024 mode, with a security level roughly equivalent to AES-256.                   |
//! | wasm      | For compiling to WASM targets. |
//! | zeroize | This will zero out the key exchange structs on drop using the [zeroize](https://docs.rs/zeroize/latest/zeroize/) crate |
//! | std | Enable the standard library |
//! 
//! ## Usage 
//! 
//! 
//! ```
//! use evo_core_crypto_kyber::*;
//! ```
//! 
//! ##### Key Encapsulation
//! ```
//! # use evo_core_crypto_kyber::*;
//! # fn main() -> Result<(),KyberError> {
//! # let mut rng = rand::thread_rng();
//! // Generate Keypair
//! let keys_bob = keypair(&mut rng);
//! 
//! // Alice encapsulates a shared secret using Bob's public key
//! let (ciphertext, shared_secret_alice) = encapsulate(&keys_bob.public, &mut rng)?;
//! 
//! // Bob decapsulates a shared secret using the ciphertext sent by Alice 
//! let shared_secret_bob = decapsulate(&ciphertext, &keys_bob.secret)?;
//! 
//! assert_eq!(shared_secret_alice, shared_secret_bob);
//! # Ok(()) }
//! ```
//! 
//! Higher level functions offering unilateral or mutual authentication
//! 
//! #### Unilaterally Authenticated Key Exchange
//! ```
//! # use evo_core_crypto_kyber::*;
//! # fn main() -> Result<(),KyberError> {
//! let mut rng = rand::thread_rng();
//! 
//! // Initialize the key exchange structs
//! let mut alice = Uake::new();
//! let mut bob = Uake::new();
//! 
//! // Generate Keypairs
//! let alice_keys = keypair(&mut rng);
//! let bob_keys = keypair(&mut rng);
//! 
//! // Alice initiates key exchange
//! let client_init = alice.client_init(&bob_keys.public, &mut rng);
//! 
//! // Bob authenticates and responds
//! let server_send = bob.server_receive(
//!   client_init, &bob_keys.secret, &mut rng
//! )?;
//! 
//! // Alice decapsulates the shared secret
//! alice.client_confirm(server_send)?;
//! 
//! // Both key exchange structs now have the shared secret
//! assert_eq!(alice.shared_secret, bob.shared_secret);
//! # Ok(()) }
//! ```
//! 
//! #### Mutually Authenticated Key Exchange
//! Follows the same workflow except Bob requires Alice's public key
//! 
//! ```
//! # use evo_core_crypto_kyber::*;
//! # use evo_core_id::UId;
//! # fn main() -> Result<(),KyberError> {
//! # let mut rng = rand::thread_rng();
//! let mut alice = Ake::new();
//! let mut bob = Ake::new();
//!
//! let alice_keys = keypair(&mut rng);
//! let bob_keys = keypair(&mut rng);
//!
//! let id_alice = UId::id();
//! let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
//!
//! let (client_id, server_send, temp_key) = bob.on_server_receive(
//!   client_init, &alice_keys.public, &bob_keys.secret, &mut rng
//! )?;
//!
//! alice.do_client_confirm(server_send, &alice_keys.secret)?;
//!
//! assert_eq!(alice.shared_secret, bob.shared_secret);
//! # Ok(()) }
//! ```
//! 
//! 
//! ## Errors
//! The [KyberError](enum.KyberError.html) enum handles errors. It has two variants:
//! 
//! * **InvalidInput** - One or more byte inputs to a function are incorrectly sized. A likely cause of 
//! this is two parties using different security levels while trying to negotiate a key exchange.
//! 
//! * **Decapsulation** - The ciphertext was unable to be authenticated. The shared secret was not decapsulated  

#![cfg_attr(not(feature ="std"), no_std)]
#![allow(clippy::many_single_char_names)]
extern crate alloc;

// Prevent usage of mutually exclusive features
#[cfg(all(feature = "kyber1024", feature = "kyber512"))]
compile_error!("Only one security level can be specified");

mod reference;
use reference::*;

#[cfg(feature = "wasm")]
mod wasm;

mod api;
mod error;
mod kem;
mod kex;
mod params;
mod rng;
mod symmetric;

pub use api::*;
pub use kex::*;
pub use params::*;
pub use error::KyberError;
pub use rand_core::{RngCore, CryptoRng};

// Feature hack to expose private functions for the Known Answer Tests
// and fuzzing. Will fail to compile if used outside `cargo test` or 
// the fuzz binaries.
#[cfg(any(kyber_kat, fuzzing, feature = "benchmarking"))]
pub use kem::*;
