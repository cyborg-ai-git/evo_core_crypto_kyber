


<img src="https://avatars.githubusercontent.com/u/129898917?v=4" alt="cyborgai" width="256" height="256">

---

## [CyborgAI](https://github.com/cyborg-ai-git) (https://github.com/cyborg-ai-git)

---

# evo_core_crypto_kyber

> forked from [bwesterb/argyle-kyber](https://github.com/bwesterb/argyle-kyber)
---

> ⚠️ **BETA DISCLAIMER**: **evo_core_crypto_kyber** is currently in beta version. Use at your own risk. Features may be unstable and subject to change without notice. This software is provided "as is" without warranty of any kind.

> While much care has been taken porting from the C reference codebase, this library has not undergone any third-party security auditing nor can any guarantees be made about the potential for underlying vulnerabilities in LWE cryptography or potential side-channel attacks arising from this implementation.

> Kyber is relatively new, it is advised to use it in a hybrid key exchange system alongside a traditional algorithm like X25519 rather than by itself.

> For further reading the IETF have a draft construction for hybrid key exchange in TLS 1.3:

> https://www.ietf.org/archive/id/draft-ietf-tls-hybrid-design-04.html

> You can also see how such a system is implemented [here](https://github.com/openssh/openssh-portable/blob/a2188579032cf080213a78255373263466cb90cc/kexsntrup761x25519.c) in C by OpenSSH

> ⚠️  Please use at your own risk.

---

## License

### License Terms
> ####  Apache License Version 2.0, January 2004
---

<p align="center">
  <img src="documentation/data/kyber.png"/>
</p>

---

# Kyber

A rust implementation of the Kyber algorithm, a KEM standardised by the NIST Post-Quantum Standardization Project.

This library:
* Is no_std compatible and needs no allocator, suitable for embedded devices. 
* Reference files contain no unsafe code and are written in pure rust.
* Compiles to WASM using wasm-bindgen and has a ready-to-use binary published on NPM.
* Features optimized AKE protocol with split server functions for flexible client identity management.
* Memory-efficient design using zero-copy operations where possible.


See the [**features**](#features) section for different options regarding security levels and modes of operation. The default security setting is kyber1024.

It is recommended to use Kyber in a hybrid system alongside a traditional key exchange algorithm such as X25519. 

Please also read the [**security considerations**](#security-considerations) before use.

**Minimum Supported Rust Version: 1.56.0**

---


---
## Installation

```shell
cargo add --git https://github.com/cyborg-ai-git/evo_core_crypto_kyber
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
evo_core_crypto_kyber = { git = "https://github.com/cyborg-ai-git/evo_core_crypto_kyber" }
```

## Usage 

```rust
use evo_core_crypto_kyber::*;
use evo_core_id::UId; // For AKE with client identification
```

---

### Key Encapsulation

```rust
// Generate Keypair
let keys_bob = keypair(&mut rng);

// Alice encapsulates a shared secret using Bob's public key
let (ciphertext, shared_secret_alice) = encapsulate(&keys_bob.public, &mut rng)?;

// Bob decapsulates a shared secret using the ciphertext sent by Alice 
let shared_secret_bob = decapsulate(&ciphertext, &keys_bob.secret)?;

assert_eq!(shared_secret_alice, shared_secret_bob);
```

---

### Unilaterally Authenticated Key Exchange
```rust
let mut rng = rand::thread_rng();

// Initialize the key exchange structs
let mut alice = Uake::new();
let mut bob = Uake::new();

// Generate Bob's Keypair
let bob_keys = keypair(&mut rng);

// Alice initiates key exchange
let client_init = alice.client_init(&bob_keys.public, &mut rng);

// Bob authenticates and responds
let server_response = bob.server_receive(
  client_init, &bob_keys.secret, &mut rng
)?;

// Alice decapsulates the shared secret
alice.client_confirm(server_response)?;

// Both key exchange structs now have the same shared secret
assert_eq!(alice.shared_secret, bob.shared_secret);
```

---

### Mutually Authenticated Key Exchange

The AKE protocol supports a split server workflow for scenarios where you need to extract the client ID before looking up the client's public key from an external source:

```rust
use evo_core_id::UId;

let mut alice = Ake::new();
let mut bob = Ake::new();

let alice_keys = keypair(&mut rng);
let bob_keys = keypair(&mut rng);
let id_alice = UId::id();

// Alice initiates with her ID embedded in the message
let client_send = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);

// Bob extracts client ID and temp key (without needing Alice's public key yet)
let (id_client, temp_key) = bob.on_server_receive(&client_send, &bob_keys.secret)?;

// Now you can use id_client to look up alice_keys.public from your database/registry
assert_eq!(id_alice, id_client);

// Bob completes the key exchange with Alice's public key
let server_send = bob.do_server_send(&client_send, &alice_keys.public, &bob_keys.secret, &mut rng)?;

// Alice confirms and derives the shared secret
alice.do_client_confirm(server_send, &alice_keys.secret)?;

assert_eq!(alice.shared_secret, bob.shared_secret);
```

#### Performance Optimizations

The AKE implementation includes several optimizations:

- **Memory Efficient**: Functions use references instead of copying 3200-byte messages, saving ~6.4KB per operation
- **Split Server Functions**: Allows extracting client identity before public key lookup
- **Zero-Copy Design**: Minimizes memory allocations during key exchange

#### AKE Performance Benchmarks

| Function | Average Time |
|----------|-------------|
| Keypair Generation | ~90 µs |
| Client Send | ~253 µs |
| Server Receive | ~130 µs |
| Server Send | ~660 µs |
| Client Confirm | ~645 µs |
| **Full Round Trip** | **~1.46 ms** |

---

## Examples

The library includes comprehensive examples demonstrating different use cases:

```bash
# Run the AKE example with client identification
cargo run --example example_ake

# Run the basic KEM example  
cargo run --example example_kem
```

The AKE example demonstrates the complete workflow including:
- Client identity embedding and extraction
- Split server-side processing
- Memory-efficient message handling
- Shared secret derivation and verification

---

## Errors
The KyberError enum has two variants:

* **InvalidInput** - One or more inputs to a function are incorrectly sized. A possible cause of this is two parties using different security levels while trying to negotiate a key exchange.

* **Decapsulation** - The ciphertext was unable to be authenticated. The shared secret was not decapsulated.

---

## Features

If no security level is specified then kyber1024 is used by default, providing the highest security level roughly equivalent to AES-256. All features can be combined as needed. For example:

```toml
[dependencies]
evo_core_crypto_kyber = { git = "https://github.com/cyborg-ai-git/evo_core_crypto_kyber", features = ["kyber512"] }
```


| Feature   | Description |
|-----------|------------|
| std | Enable the standard library |
| kyber512  | Enables kyber512 mode, with a security level roughly equivalent to AES-128.|
| kyber768  | Enables kyber768 mode, with a security level roughly equivalent to AES-192.|
| kyber1024 | Enables kyber1024 mode, with a security level roughly equivalent to AES-256. **This is the default security level.** A compile-time error is raised if more than one security level is specified.|
| wasm | For compiling to WASM targets|
| zeroize | This will zero out the key exchange structs on drop using the [zeroize](https://docs.rs/zeroize/latest/zeroize/) crate |
| benchmarking |  Enables the criterion benchmarking suite |
---

## Testing

The [run_all_tests](tests/run_all_tests.sh) script will traverse all possible codepaths by running a matrix of the security levels, variants and crate features.

Known Answer Tests require deterministic rng seeds, enable `kyber_kat` in `RUSTFLAGS`to use them. 
Using this outside of `cargo test` will result in a compile-time error. 
The test vector files are quite large, you will need to build them yourself from the C reference code. 
There's a helper script to do this [here](./tests/KAT/build_kats.sh). 

```bash
# This example runs the basic tests for kyber1024 (default)
cargo test
```

See the [testing readme](./tests/readme.md) for more comprehensive info.

---

## Benchmarking

Uses criterion for benchmarking. If you have GNUPlot installed it will generate statistical graphs in `./target/criterion/`.

You will need to enable the `benchmarking` feature.

```bash
# Run all benchmarks
cargo bench --features benchmarking

# Run specific benchmark suites
cargo bench --bench bench_kyber --features benchmarking      # Core KEM operations
cargo bench --bench bench_kyber_ake --features benchmarking  # AKE protocol functions
```

Available benchmark suites:
- **bench_kyber**: Core Kyber KEM operations (keypair, encapsulate, decapsulate)
- **bench_kyber_ake**: AKE protocol functions (client_send, server_receive, server_send, client_confirm, full round trip)

See the [benchmarking readme](./benches/readme.md) for information on correct usage.

---

## Fuzzing

The fuzzing suite uses honggfuzz, installation and instructions are on the [fuzzing](./fuzz/readme.md) page. 

---



## About

Kyber is an IND-CCA2-secure key encapsulation mechanism (KEM), whose security is based on the hardness of solving the learning-with-errors (LWE) problem over module lattices. It is the final standardised algorithm resulting from the [NIST post-quantum cryptography project](https://csrc.nist.gov/Projects/Post-Quantum-Cryptography).

The official website: https://pq-crystals.org/kyber/

Authors of the Kyber Algorithm: 

* Roberto Avanzi, ARM Limited (DE)
* Joppe Bos, NXP Semiconductors (BE)
* Léo Ducas, CWI Amsterdam (NL)
* Eike Kiltz, Ruhr University Bochum (DE)
* Tancrède Lepoint, SRI International (US)
* Vadim Lyubashevsky, IBM Research Zurich (CH)
* John M. Schanck, University of Waterloo (CA)
* Peter Schwabe, Radboud University (NL)
* Gregor Seiler, IBM Research Zurich (CH)
* Damien Stehle, ENS Lyon (FR)

---

### Contributing 

Contributions welcome. For pull requests create a feature fork and submit it to the development branch. More information is available on the [contributing page](./contributing.md)

---

### Alternatives

The PQClean project has rust bindings for their Kyber C codebase:

https://github.com/rustpq/pqcrypto

