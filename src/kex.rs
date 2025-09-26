use crate::{KyberError, kem::*, params::*, symmetric::kdf};
use alloc::vec::Vec;
use evo_framework::TypeID;
use rand_core::{CryptoRng, RngCore};
#[cfg(feature = "zeroize")]
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Unilateral Key Exchange Initiation Byte Length
pub const UAKE_INIT_BYTES: usize = KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES;
/// Unilateral Key Exchange Response Byte Length
pub const UAKE_RESPONSE_BYTES: usize = KYBER_CIPHERTEXTBYTES;
/// Mutual Key Exchange Initiation Byte Length (includes 64 bytes for encrypted client ID + temp key)
pub const AKE_INIT_BYTES: usize = KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES + 64;
/// Mutual Key Exchange Response Byte Length
pub const AKE_RESPONSE_BYTES: usize = 2 * KYBER_CIPHERTEXTBYTES;

/// Result of encapsulating a public key which includes the ciphertext and shared secret
pub type Encapsulated = Result<([u8; KYBER_CIPHERTEXTBYTES], [u8; KYBER_SSBYTES]), KyberError>;
/// The result of  decapsulating a ciphertext which produces a shared secret when confirmed
pub type Decapsulated = Result<[u8; KYBER_SSBYTES], KyberError>;
/// Kyber public key
pub type PublicKey = [u8; KYBER_PUBLICKEYBYTES];
/// Kyber secret key
pub type SecretKey = [u8; KYBER_SECRETKEYBYTES];
/// Kyber Shared Secret
pub type SharedSecret = [u8; KYBER_SSBYTES];
/// Bytes to send when initiating a unilateral key exchange
pub type UakeSendInit = [u8; UAKE_INIT_BYTES];
/// Bytes to send when responding to a unilateral key exchange
pub type UakeSendResponse = [u8; UAKE_RESPONSE_BYTES];
/// Bytes to send when initiating a mutual key exchange
pub type AkeSendInit = [u8; AKE_INIT_BYTES];
/// Bytes to send when responding to a mutual key exchange
pub type AkeSendResponse = [u8; AKE_RESPONSE_BYTES];
//==================================================================================================
// Ephemeral keys
type TempKey = [u8; KYBER_SSBYTES];
type Eska = [u8; KYBER_SECRETKEYBYTES];
//==================================================================================================
/// Used for unilaterally authenticated key exchange between two parties.
///
/// ```
/// # use evo_core_crypto_kyber::*;
/// # fn main() -> Result<(),KyberError> {
/// let mut rng = rand::thread_rng();
///
/// let mut alice = Uake::new();
/// let mut bob = Uake::new();
/// let bob_keys = keypair(&mut rng);
///
/// let client_init = alice.client_init(&bob_keys.public, &mut rng);
/// let server_send = bob.server_receive(client_init, &bob_keys.secret, &mut rng)?;
/// let client_confirm = alice.client_confirm(server_send);
///
/// assert_eq!(alice.shared_secret, bob.shared_secret);
/// # Ok(()) }
//--------------------------------------------------------------------------------------------------
#[cfg_attr(feature = "zeroize", derive(Zeroize, ZeroizeOnDrop))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Uake {
    /// The resulting shared secret from a key exchange
    pub shared_secret: SharedSecret,
    /// Sent when initiating a key exchange
    send_a: UakeSendInit,
    /// Response to a key exchange initiation
    send_b: UakeSendResponse,
    // Ephemeral keys
    temp_key: TempKey,
    eska: Eska,
}
//--------------------------------------------------------------------------------------------------
impl Default for Uake {
    fn default() -> Self {
        Uake {
            shared_secret: [0u8; KYBER_SSBYTES],
            send_a: [0u8; UAKE_INIT_BYTES],
            send_b: [0u8; UAKE_RESPONSE_BYTES],
            temp_key: [0u8; KYBER_SSBYTES],
            eska: [0u8; KYBER_SECRETKEYBYTES],
        }
    }
}
//--------------------------------------------------------------------------------------------------
impl Uake {
    /// Builds new UAKE struct
    /// ```
    /// # use evo_core_crypto_kyber::Uake;
    /// let mut kex = Uake::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Initiates a Unilaterally Authenticated Key Exchange.
    /// ```
    /// # use evo_core_crypto_kyber::*;
    /// # fn main() -> Result<(),KyberError> {
    /// let mut rng = rand::thread_rng();
    /// let mut alice = Uake::new();
    /// let bob_keys = keypair(&mut rng);
    /// let client_init = alice.client_init(&bob_keys.public, &mut rng);
    /// # Ok(()) }
    /// ```
    pub fn client_init<R>(&mut self, pubkey: &PublicKey, rng: &mut R) -> UakeSendInit
    where
        R: CryptoRng + RngCore,
    {
        uake_init_a(
            &mut self.send_a,
            &mut self.temp_key,
            &mut self.eska,
            pubkey,
            rng,
        );
        self.send_a
    }

    /// Handles the output of a `client_init()` request
    /// ```
    /// # use evo_core_crypto_kyber::*;
    /// # fn main() -> Result<(),KyberError> {
    /// # let mut rng = rand::thread_rng();
    /// let mut alice = Uake::new();
    /// let mut bob = Uake::new();
    /// let mut bob_keys = keypair(&mut rng);
    /// let client_init = alice.client_init(&bob_keys.public, &mut rng);
    /// let server_send = bob.server_receive(client_init, &bob_keys.secret, &mut rng)?;
    /// # Ok(()) }
    pub fn server_receive<R>(
        &mut self,
        send_a: UakeSendInit,
        secretkey: &SecretKey,
        rng: &mut R,
    ) -> Result<UakeSendResponse, KyberError>
    where
        R: CryptoRng + RngCore,
    {
        uake_shared_b(
            &mut self.send_b,
            &mut self.shared_secret,
            &send_a,
            secretkey,
            rng,
        )?;
        Ok(self.send_b)
    }

    /// Decapsulates and authenticates the shared secret from the output of
    /// `server_receive()`
    /// ```
    /// # use evo_core_crypto_kyber::*;
    /// # fn main() -> Result<(),KyberError> {
    /// # let mut rng = rand::thread_rng();
    /// # let mut alice = Uake::new();
    /// # let mut bob = Uake::new();
    /// # let bob_keys = keypair(&mut rng);
    /// let client_init = alice.client_init(&bob_keys.public, &mut rng);
    /// let server_send = bob.server_receive(client_init, &bob_keys.secret, &mut rng)?;
    /// let client_confirm = alice.client_confirm(server_send);
    /// assert_eq!(alice.shared_secret, bob.shared_secret);
    /// # Ok(()) }
    pub fn client_confirm(&mut self, send_b: UakeSendResponse) -> Result<(), KyberError> {
        uake_shared_a(&mut self.shared_secret, &send_b, &self.temp_key, &self.eska)?;
        Ok(())
    }
}
//--------------------------------------------------------------------------------------------------
/// Used for mutually authenticated key exchange between two parties.
///
/// # Example:
/// ```
/// # use evo_core_crypto_kyber::*;
/// # use evo_core_id::UId;
/// # fn main() -> Result<(),KyberError> {
/// let mut rng = rand::thread_rng();
///
/// let mut alice = Ake::new();
/// let mut bob = Ake::new();
///
/// let alice_keys = keypair(&mut rng);
/// let bob_keys = keypair(&mut rng);
///
/// let id_alice = UId::id();
/// let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
/// let (client_id, server_send, temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng)?;
/// let client_confirm = alice.do_client_confirm(server_send, &alice_keys.secret);
///
/// assert_eq!(alice.shared_secret, bob.shared_secret);
/// # Ok(()) }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "zeroize", derive(Zeroize, ZeroizeOnDrop))]
pub struct Ake {
    /// The resulting shared secret from a key exchange
    pub shared_secret: SharedSecret,
    /// Sent when initiating a key exchange
    send_a: AkeSendInit,
    /// Response to a key exchange initiation
    send_b: AkeSendResponse,
    // Ephemeral keys
    pub temp_key: TempKey,
    pub eska: Eska,
}
//--------------------------------------------------------------------------------------------------
impl Default for Ake {
    fn default() -> Self {
        Ake {
            shared_secret: [0u8; KYBER_SSBYTES],
            send_a: [0u8; AKE_INIT_BYTES],
            send_b: [0u8; AKE_RESPONSE_BYTES],
            temp_key: [0u8; KYBER_SSBYTES],
            eska: [0u8; KYBER_SECRETKEYBYTES],
        }
    }
}
//--------------------------------------------------------------------------------------------------
impl Ake {
    /// Builds a new AKE struct
    /// ```
    /// # use evo_core_crypto_kyber::Ake;
    /// let mut kex = Ake::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Initiates a Mutually Authenticated Key Exchange.
    /// ```
    /// # use evo_core_crypto_kyber::*;
    /// # use evo_core_id::UId;
    /// # fn main() -> Result<(),KyberError> {
    /// let mut rng = rand::thread_rng();
    /// let mut alice = Ake::new();
    /// let bob_keys = keypair(&mut rng);
    /// let id_alice = UId::id();
    /// let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
    /// # Ok(()) }
    /// ```
    pub fn do_client_send<R>(
        &mut self,
        pk_server: &PublicKey,
        id_client: TypeID,
        rng: &mut R,
    ) -> AkeSendInit
    where
        R: CryptoRng + RngCore,
    {
        ake_init_a_with_id(
            &mut self.send_a,
            &mut self.temp_key,
            &mut self.eska,
            pk_server,
            &id_client,
            rng,
        );
        self.send_a
    }

    /// Handles and authenticates the output of a `client_init()` request
    /// ```
    /// # use evo_core_crypto_kyber::*;
    /// # use evo_core_id::UId;
    /// # fn main() -> Result<(),KyberError> {
    /// # let mut rng = rand::thread_rng();
    /// let mut alice = Ake::new();
    /// let mut bob = Ake::new();
    /// let alice_keys = keypair(&mut rng);
    /// let bob_keys = keypair(&mut rng);
    /// let id_alice = UId::id();
    /// let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
    /// let (client_id, server_send, temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng)?;
    /// # Ok(()) }
    pub fn on_server_receive<R>(
        &mut self,
        ake_send_a: AkeSendInit,
        pub_key: &PublicKey,
        secret_key: &SecretKey,
        rng: &mut R,
    ) -> Result<(TypeID, AkeSendResponse, Vec<u8>), KyberError>
    where
        R: CryptoRng + RngCore,
    {
        let (client_id, alice_temp_key) = ake_shared_b_with_id(
            &mut self.send_b,
            &mut self.shared_secret,
            &ake_send_a,
            secret_key,
            pub_key,
            rng,
        )?;
        Ok((client_id, self.send_b, alice_temp_key.to_vec()))
    }

    /// Decapsulates and authenticates the shared secret from the output of
    /// `server_receive()`
    /// ```
    /// # use evo_core_crypto_kyber::*;
    /// # use evo_core_id::UId;
    /// # fn main() -> Result<(),KyberError> {
    /// # let mut rng = rand::thread_rng();
    /// # let mut alice = Ake::new();
    /// # let mut bob = Ake::new();
    /// # let alice_keys = keypair(&mut rng);
    /// # let bob_keys = keypair(&mut rng);
    /// # let id_alice = UId::id();
    /// # let client_init = alice.do_client_send(&bob_keys.public, id_alice, &mut rng);
    /// let (client_id, server_send, temp_key) = bob.on_server_receive(client_init, &alice_keys.public, &bob_keys.secret, &mut rng)?;
    /// let client_confirm = alice.do_client_confirm(server_send, &alice_keys.secret);
    /// assert_eq!(alice.shared_secret, bob.shared_secret);
    /// # Ok(()) }
    pub fn do_client_confirm(
        &mut self,
        send_b: AkeSendResponse,
        secretkey: &SecretKey,
    ) -> Result<(), KyberError> {
        ake_shared_a(
            &mut self.shared_secret,
            &send_b,
            &self.temp_key,
            &self.eska,
            secretkey,
        )?;
        Ok(())
    }
}
//--------------------------------------------------------------------------------------------------
// Unilaterally Authenticated Key Exchange initiation
fn uake_init_a<R>(send: &mut [u8], tk: &mut [u8], sk: &mut [u8], pkb: &[u8], rng: &mut R)
where
    R: CryptoRng + RngCore,
{
    crypto_kem_keypair(send, sk, rng, None);
    crypto_kem_enc(&mut send[KYBER_PUBLICKEYBYTES..], tk, pkb, rng, None);
}
//--------------------------------------------------------------------------------------------------
// Unilaterally authenticated key exchange computation by Bob
fn uake_shared_b<R>(
    send: &mut [u8],
    k: &mut [u8],
    recv: &[u8],
    skb: &[u8],
    rng: &mut R,
) -> Result<(), KyberError>
where
    R: CryptoRng + RngCore,
{
    let mut buf = [0u8; 2 * KYBER_SYMBYTES];
    crypto_kem_enc(send, &mut buf, recv, rng, None);
    crypto_kem_dec(
        &mut buf[KYBER_SYMBYTES..],
        &recv[KYBER_PUBLICKEYBYTES..],
        skb,
    );
    kdf(k, &buf, 2 * KYBER_SYMBYTES);
    Ok(())
}
//--------------------------------------------------------------------------------------------------
// Unilaterally authenticated key exchange computation by Alice
fn uake_shared_a(k: &mut [u8], recv: &[u8], tk: &[u8], sk: &[u8]) -> Result<(), KyberError> {
    let mut buf = [0u8; 2 * KYBER_SYMBYTES];
    crypto_kem_dec(&mut buf, recv, sk);
    buf[KYBER_SYMBYTES..].copy_from_slice(&tk[..]);
    kdf(k, &buf, 2 * KYBER_SYMBYTES);
    Ok(())
}
//--------------------------------------------------------------------------------------------------
// Authenticated key exchange initiation by Alice
fn ake_init_a<R>(send: &mut [u8], temp_key: &mut [u8], sk: &mut [u8], pkb: &[u8], rng: &mut R)
where
    R: CryptoRng + RngCore,
{
    crypto_kem_keypair(send, sk, rng, None);
    crypto_kem_enc(&mut send[KYBER_PUBLICKEYBYTES..], temp_key, pkb, rng, None);
}
//--------------------------------------------------------------------------------------------------
// Mutually authenticated key exchange computation by Bob
fn ake_shared_b<R>(
    send: &mut [u8],
    k: &mut [u8],
    recv: &[u8],
    skb: &[u8],
    pka: &[u8],
    rng: &mut R,
) -> Result<TempKey, KyberError>
where
    R: CryptoRng + RngCore,
{
    let mut buf = [0u8; 3 * KYBER_SYMBYTES];
    let mut alice_temp_key = [0u8; KYBER_SSBYTES];
    
    crypto_kem_enc(send, &mut buf, recv, rng, None);
    crypto_kem_enc(
        &mut send[KYBER_CIPHERTEXTBYTES..],
        &mut buf[KYBER_SYMBYTES..],
        pka,
        rng,
        None,
    );
    
    // Extract Alice's temp_key by decrypting the ciphertext portion of her message
    // The ciphertext starts at KYBER_PUBLICKEYBYTES offset in recv
    crypto_kem_dec(
        &mut alice_temp_key,
        &recv[KYBER_PUBLICKEYBYTES..],
        skb,
    );
    
    // Also decrypt for the shared secret computation
    crypto_kem_dec(
        &mut buf[2 * KYBER_SYMBYTES..],
        &recv[KYBER_PUBLICKEYBYTES..],
        skb,
    );
    
    kdf(k, &buf, 3 * KYBER_SYMBYTES);
    Ok(alice_temp_key)
}
//--------------------------------------------------------------------------------------------------
// Mutually authenticated key exchange computation by Alice
fn ake_shared_a(
    k: &mut [u8],
    recv: &[u8],
    tk: &[u8],
    sk: &[u8],
    ska: &[u8],
) -> Result<(), KyberError> {
    let mut buf = [0u8; 3 * KYBER_SYMBYTES];
    crypto_kem_dec(&mut buf, recv, sk);
    crypto_kem_dec(
        &mut buf[KYBER_SYMBYTES..],
        &recv[KYBER_CIPHERTEXTBYTES..],
        ska,
    );
    buf[2 * KYBER_SYMBYTES..].copy_from_slice(&tk[..]);
    kdf(k, &buf, 3 * KYBER_SYMBYTES);
    Ok(())
}
//--------------------------------------------------------------------------------------------------
// Authenticated key exchange initiation by Alice with client ID
fn ake_init_a_with_id<R>(
    send: &mut [u8], 
    temp_key: &mut [u8], 
    sk: &mut [u8], 
    pkb: &[u8], 
    id_client: &[u8; 32],
    rng: &mut R
)
where
    R: CryptoRng + RngCore,
{
    // Generate Alice's ephemeral keypair
    crypto_kem_keypair(send, sk, rng, None);
    
    // Encrypt temp_key with Bob's public key (original AKE behavior)
    crypto_kem_enc(&mut send[KYBER_PUBLICKEYBYTES..], temp_key, pkb, rng, None);
    
    // Create payload: temp_key (32 bytes) + id_client (32 bytes) = 64 bytes
    let mut payload = [0u8; 64];
    payload[..32].copy_from_slice(temp_key);
    payload[32..].copy_from_slice(id_client);
    
    // Use temp_key as encryption key for the additional payload
    // Simple XOR encryption (in practice, use a proper symmetric cipher)
    for i in 0..64 {
        payload[i] ^= temp_key[i % 32];
    }
    
    // Store the encrypted payload in the additional space
    if send.len() >= KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES + 64 {
        send[KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES..KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES + 64]
            .copy_from_slice(&payload);
    }
}
//--------------------------------------------------------------------------------------------------
// Mutually authenticated key exchange computation by Bob with client ID extraction
fn ake_shared_b_with_id<R>(
    send: &mut [u8],
    k: &mut [u8],
    recv: &[u8],
    skb: &[u8],
    pka: &[u8],
    rng: &mut R,
) -> Result<([u8; 32], TempKey), KyberError>
where
    R: CryptoRng + RngCore,
{
    let mut buf = [0u8; 3 * KYBER_SYMBYTES];
    let mut alice_temp_key = [0u8; KYBER_SSBYTES];
    
    crypto_kem_enc(send, &mut buf, recv, rng, None);
    crypto_kem_enc(
        &mut send[KYBER_CIPHERTEXTBYTES..],
        &mut buf[KYBER_SYMBYTES..],
        pka,
        rng,
        None,
    );
    
    // Extract Alice's temp_key by decrypting the ciphertext portion of her message
    // The ciphertext starts at KYBER_PUBLICKEYBYTES offset in recv
    crypto_kem_dec(
        &mut alice_temp_key,
        &recv[KYBER_PUBLICKEYBYTES..],
        skb,
    );
    
    // Also decrypt for the shared secret computation
    crypto_kem_dec(
        &mut buf[2 * KYBER_SYMBYTES..],
        &recv[KYBER_PUBLICKEYBYTES..],
        skb,
    );
    
    // Extract the encrypted payload from the additional space
    let mut payload = [0u8; 64];
    if recv.len() >= KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES + 64 {
        payload.copy_from_slice(
            &recv[KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES..KYBER_PUBLICKEYBYTES + KYBER_CIPHERTEXTBYTES + 64]
        );
    }
    
    // Decrypt the payload using XOR with Alice's temp_key
    for i in 0..64 {
        payload[i] ^= alice_temp_key[i % 32];
    }
    
    // Extract temp_key and client_id from decrypted payload
    let mut extracted_temp_key = [0u8; 32];
    let mut client_id = [0u8; 32];
    extracted_temp_key.copy_from_slice(&payload[..32]);
    client_id.copy_from_slice(&payload[32..]);
    
    // Verify that the extracted temp_key matches what we decrypted
    // (This is a consistency check)
    if extracted_temp_key != alice_temp_key {
        return Err(KyberError::InvalidInput);
    }
    
    kdf(k, &buf, 3 * KYBER_SYMBYTES);
    Ok((client_id, alice_temp_key))
}
//==================================================================================================
