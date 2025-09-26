/// The security level of Kyber
/// 
/// Defaults to 3 (kyber768), will be 2 or 4 respectively when
/// kyber512 or kyber1024 are selected with feature flags.
/// 
/// * Kyber-512 aims at security roughly equivalent to AES-128
/// * Kyber-768 aims at security roughly equivalent to AES-192
/// * Kyber-1024 aims at security roughly equivalent to AES-256
pub const KYBER_K: usize = if cfg!(feature = "kyber512") {
    2
} else if cfg!(feature = "kyber1024") {
    4
} else {
    3
};

pub(crate) const KYBER_N: usize = 256;
pub(crate) const KYBER_Q: usize = 3329;

pub(crate) const KYBER_ETA1: usize = if cfg!(feature = "kyber512") { 3 } else { 2 };
pub(crate) const KYBER_ETA2: usize = 2;

// Size of the hashes and seeds
pub const KYBER_SYMBYTES: usize = 32;

/// Size of the shared key 
pub const KYBER_SSBYTES: usize =  32; 

pub(crate) const KYBER_POLYBYTES: usize = 384;
pub(crate) const KYBER_POLYVECBYTES: usize =  KYBER_K * KYBER_POLYBYTES;

#[cfg(not(feature = "kyber1024"))]
pub(crate) const KYBER_POLYCOMPRESSEDBYTES: usize =     128;
#[cfg(not(feature = "kyber1024"))]
pub(crate) const KYBER_POLYVECCOMPRESSEDBYTES: usize =  KYBER_K * 320;

#[cfg(feature = "kyber1024")]
pub(crate) const KYBER_POLYCOMPRESSEDBYTES: usize =     160;
#[cfg(feature = "kyber1024")]
pub(crate) const KYBER_POLYVECCOMPRESSEDBYTES: usize = KYBER_K * 352;

pub(crate) const KYBER_INDCPA_PUBLICKEYBYTES: usize = KYBER_POLYVECBYTES + KYBER_SYMBYTES;
pub(crate) const KYBER_INDCPA_SECRETKEYBYTES: usize = KYBER_POLYVECBYTES;
pub(crate) const KYBER_INDCPA_BYTES: usize = KYBER_POLYVECCOMPRESSEDBYTES + KYBER_POLYCOMPRESSEDBYTES;

/// Size in bytes of the Kyber public key
pub const KYBER_PUBLICKEYBYTES: usize = KYBER_INDCPA_PUBLICKEYBYTES;
/// Size in bytes of the Kyber secret key 
pub const KYBER_SECRETKEYBYTES: usize =  KYBER_INDCPA_SECRETKEYBYTES +  KYBER_INDCPA_PUBLICKEYBYTES + 2*KYBER_SYMBYTES; 
/// Size in bytes of the Kyber ciphertext
pub const KYBER_CIPHERTEXTBYTES: usize =  KYBER_INDCPA_BYTES;
