use crate::{
  params::*,
  ntt::*,
  reduce::*,
  cbd::*,
  symmetric::*
};

#[derive(Clone)]
pub struct Poly {
  pub coeffs: [i16; KYBER_N]  
}

impl Copy for Poly {}

impl Default for Poly {
  fn default() -> Self {
    Poly {
      coeffs: [0i16; KYBER_N]
    }
  }
}

// new() is nicer
impl Poly {
  pub fn new() -> Self {
    Self::default()
  }
}

// Name:        poly_compress
//
// Description: Compression and subsequent serialization of a polynomial
//
// Arguments:   - [u8] r: output byte array (needs space for KYBER_POLYCOMPRESSEDBYTES bytes)
//              - const poly *a:    input polynomial
pub fn poly_compress(r: &mut[u8], a: Poly)
{
  let mut t = [0u8; 8];
  let mut k = 0usize;
  let mut u: i16;

  // Compress_q(x, d) = ⌈(2ᵈ/q)x⌋ mod⁺ 2ᵈ
  //                  = ⌊(2ᵈ/q)x+½⌋ mod⁺ 2ᵈ
  //                  = ⌊((x << d) + q/2) / q⌋ mod⁺ 2ᵈ
  //                  = DIV((x << d) + q/2, q) & ((1<<d) - 1)
  //
  // We approximate DIV(x, q) by computing (x*a)>>e, where a/(2^e) ≈ 1/q.
  // For d in {10,11} we use 20,642,678/2^36, which computes division by x/q
  // correctly for 0 ≤ x < 41,522,616, which fits (q << 11) + q/2 comfortably.
  // For d in {4,5} we use 315/2^20, which doesn't compute division by x/q
  // correctly for all inputs, but it's close enough that the end result
  // of the compression is correct. The advantage is that we do not need
  // to use a 64-bit intermediate value.

  match KYBER_POLYCOMPRESSEDBYTES {
    128 => {
      for i in 0..KYBER_N/8 {
        for j in 0..8 {
          // map to positive standard representatives
          u = a.coeffs[8*i+j];
          u += (u >> 15) & KYBER_Q as i16;
          let mut tmp : u32 = (((u as u16) << 4) + KYBER_Q as u16 /2) as u32;
          tmp *= 315;
          tmp >>= 20;
          t[j] = ((tmp as u16) & 15) as u8;
        }
        r[k]   = t[0] | (t[1] << 4);
        r[k+1] = t[2] | (t[3] << 4);
        r[k+2] = t[4] | (t[5] << 4);
        r[k+3] = t[6] | (t[7] << 4);
        k += 4;
      }
    },
    160 => {
      for i in 0..(KYBER_N/8) {
        for j in 0..8 {
          // map to positive standard representatives
          u = a.coeffs[8*i+j];
          u += (u >> 15) & KYBER_Q as i16;
          let mut tmp : u32 = ((u as u32) << 5) + KYBER_Q as u32/2;
          tmp *= 315;
          tmp >>= 20;
          t[j] = ((tmp as u16) & 31) as u8;
        }
        r[k]   =  t[0]       | (t[1] << 5);
        r[k+1] = (t[1] >> 3) | (t[2] << 2) | (t[3] << 7);
        r[k+2] = (t[3] >> 1) | (t[4] << 4);
        r[k+3] = (t[4] >> 4) | (t[5] << 1) | (t[6] << 6);
        r[k+4] = (t[6] >> 2) | (t[7] << 3);
        k += 5;
      }
    },
    _ => { },
  }
}


// Name:        poly_decompress
//
// Description: De-serialization and subsequent decompression of a polynomial;
//              approximate inverse of poly_compress
//
// Arguments:   - poly *r:                output polynomial
//              - const [u8] a: input byte array (of length KYBER_POLYCOMPRESSEDBYTES bytes)
pub fn poly_decompress(r: &mut Poly, a: &[u8])
{
  match KYBER_POLYCOMPRESSEDBYTES {
    128 => {
      let mut idx = 0usize;
      for i in 0..KYBER_N/2 {
        r.coeffs[2*i+0] = ((((a[idx] & 15) as usize * KYBER_Q) + 8) >> 4) as i16;
        r.coeffs[2*i+1] = ((((a[idx] >> 4) as usize * KYBER_Q) + 8) >> 4) as i16;
        idx += 1;
      }
    },
    160 => {
      let mut idx = 0usize;
      let mut t = [0u8;8];
      for i in 0..KYBER_N/8 {
        t[0] = a[idx+0];
        t[1] = (a[idx+0] >> 5) | (a[idx+1] << 3);
        t[2] = a[idx+1] >> 2;
        t[3] = (a[idx+1] >> 7) | (a[idx+2] << 1);
        t[4] = (a[idx+2] >> 4) | (a[idx+3] << 4);
        t[5] = a[idx+3] >> 1;
        t[6] = (a[idx+3] >> 6) | (a[idx+4] << 2);
        t[7] = a[idx+4] >> 3;
        idx += 5;
        for j in 0..8 {
          r.coeffs[8*i+j] = ((((t[j] as u32) & 31)*KYBER_Q as u32 + 16) >> 5) as i16;
        }
      }
    },
    _ => { },
  }
}

// Name:        poly_tobytes
//
// Description: Serialization of a polynomial
//
// Arguments:   - [u8] r: output byte array (needs space for KYBER_POLYBYTES bytes)
//              - const poly *a:    input polynomial
pub fn poly_tobytes(r: &mut[u8], a: Poly)
{
  let (mut t0, mut t1);

  for i in 0..(KYBER_N/2) {
    // map to positive standard representatives
    t0 = a.coeffs[2*i];
    t0 += (t0 >> 15) & KYBER_Q as i16;
    t1 = a.coeffs[2*i+1];
    t1 += (t1 >> 15) & KYBER_Q as i16;
    r[3*i+0] = (t0 >> 0) as u8;
    r[3*i+1] = ((t0 >> 8) | (t1 << 4)) as u8;
    r[3*i+2] = (t1 >> 4) as u8;
  }
}

// Name:        poly_frombytes
//
// Description: De-serialization of a polynomial;
//              inverse of poly_tobytes
//
// Arguments:   - poly *r:                output polynomial
//              - const [u8] a: input byte array (of KYBER_POLYBYTES bytes)
pub fn poly_frombytes(r: &mut Poly, a: &[u8])
{
  for i in 0..(KYBER_N/2) {
    r.coeffs[2*i+0] = ((a[3*i+0] >> 0) as u16 | ((a[3*i+1] as u16) << 8) & 0xFFF) as i16;
    r.coeffs[2*i+1] = ((a[3*i+1] >> 4) as u16 | ((a[3*i+2] as u16) << 4) & 0xFFF) as i16;
  }
}

// Name:        poly_getnoise_eta1
//
// Description: Sample a polynomial deterministically from a seed and a nonce,
//              with output polynomial close to centered binomial distribution
//              with parameter KYBER_ETA1
//
// Arguments:   - poly *r:                   output polynomial
//              - const [u8] seed: input seed (pointing to array of length KYBER_SYMBYTES bytes)
//              - [u8]  nonce:       one-byte input nonce
pub fn poly_getnoise_eta1(r: &mut Poly, seed: &[u8], nonce: u8)
{
  const LENGTH: usize = KYBER_ETA1*KYBER_N/4;
  let mut buf = [0u8; LENGTH];
  prf(&mut buf, LENGTH, seed, nonce);
  poly_cbd_eta1(r, &buf);
}

// Name:        poly_getnoise_eta2
//
// Description: Sample a polynomial deterministically from a seed and a nonce,
//              with output polynomial close to centered binomial distribution
//              with parameter KYBER_ETA2
//
// Arguments:   - poly *r:                   output polynomial
//              - const [u8] seed: input seed (pointing to array of length KYBER_SYMBYTES bytes)
//              - [u8]  nonce:       one-byte input nonce
pub fn poly_getnoise_eta2(r: &mut Poly, seed: &[u8], nonce: u8)
{
  const LENGTH: usize = KYBER_ETA2*KYBER_N/4;
  let mut buf = [0u8; LENGTH];
  prf(&mut buf, LENGTH, seed, nonce);
  poly_cbd_eta2(r, &buf);
}



// Name:        poly_ntt
//
// Description: Computes negacyclic number-theoretic transform (NTT) of
//              a polynomial in place;
//              inputs assumed to be in normal order, output in bitreversed order
//
// Arguments:   - Poly r: in/output polynomial
pub fn poly_ntt(r: &mut Poly) 
{
  ntt(&mut r.coeffs);
  poly_reduce(r);
}

// Name:        poly_invntt
//
// Description: Computes inverse of negacyclic number-theoretic transform (NTT) of
//              a polynomial in place;
//              inputs assumed to be in bitreversed order, output in normal order
//
// Arguments:   - Poly a: in/output polynomial
pub fn poly_invntt_tomont(r: &mut Poly)
{
  invntt(&mut r.coeffs);
}

// Name:        poly_basemul
//
// Description: Multiplication of two polynomials in NTT domain
//
// Arguments:   - poly *r:       output polynomial
//              - const poly *a: first input polynomial
//              - const poly *b: second input polynomial
pub fn poly_basemul(r: &mut Poly, a: &Poly, b: &Poly)
{
  for i in 0..(KYBER_N/4) {
    
    basemul(
      &mut r.coeffs[4*i..], 
      &a.coeffs[4*i..],
      &b.coeffs[4*i..], 
      ZETAS[64 + i]
    );
    basemul(
      &mut r.coeffs[4*i+2..], 
      &a.coeffs[4*i+2..],
      &b.coeffs[4*i+2..],
      -(ZETAS[64 + i]));
  }
}

// Name:        poly_tomont
//
// Description: Inplace conversion of all coefficients of a polynomial 
//              from normal domain to Montgomery domain
//
// Arguments:   - poly *r:       input/output polynomial
pub fn poly_tomont(r: &mut Poly)
{
  let f = ((1u64 << 32) % KYBER_Q as u64) as i16;
  for i in 0..KYBER_N {
    let a = r.coeffs[i] as i32 * f as i32;
    r.coeffs[i] = montgomery_reduce(a);
  }
}

// Name:        poly_reduce
//
// Description: Applies Barrett reduction to all coefficients of a polynomial
//              for details of the Barrett reduction see comments in reduce.c
//
// Arguments:   - poly *r:       input/output polynomial
pub fn poly_reduce(r: &mut Poly)
{
  for i in 0..KYBER_N {
    r.coeffs[i] = barrett_reduce(r.coeffs[i]);
  }
}

// Name:        poly_add
//
// Description: Add two polynomials; no modular reduction is performed
//
// Arguments: - poly *r:       output polynomial
//            - const poly *a: first input polynomial
//            - const poly *b: second input polynomial
pub fn poly_add(r: &mut Poly, b: &Poly)
{
  for i in 0..KYBER_N {
    r.coeffs[i] += b.coeffs[i];
  }
}

// Name:        poly_sub
//
// Description: Subtract two polynomials; no modular reduction is performed
//
// Arguments: - poly *r:       output polynomial
//            - const poly *a: first input polynomial
//            - const poly *b: second input polynomial
pub fn poly_sub(r: &mut Poly, a: &Poly)
{
  for i in 0..KYBER_N {
    r.coeffs[i] = a.coeffs[i] -  r.coeffs[i];
  }
}

// Name:        poly_frommsg
//
// Description: Convert `KYBER_SYMBYTES`-byte message to polynomial
//
// Arguments:   - poly *r:                  output polynomial
//              - const [u8] msg: input message (of length KYBER_SYMBYTES)
pub fn poly_frommsg(r: &mut Poly, msg: &[u8])
{
  // Prevent compiler optimisation.
  //
  // For upstream see
  //
  // https://github.com/pq-crystals/kyber/commit/9b8d30698a3e7449aeb34e62339d4176f11e3c6c
  //
  // For method, see  https://github.com/dalek-cryptography/curve25519-dalek/pull/659
  fn black_box(value: u16) -> u16 {
    unsafe { core::ptr::read_volatile(&value) }
  }

  let mut mask;
  for i in 0..KYBER_N/8 {
    for j in 0..8 {
      mask = black_box((msg[i] as u16 >> j) & 1 ).wrapping_neg();
      r.coeffs[8*i+j] = (mask & ((KYBER_Q+1)/2) as u16) as i16;
    }
  }
}

// Name:        poly_tomsg
//
// Description: Convert polynomial to 32-byte message
//
// Arguments:   - [u8] msg: output message
//              - const poly *a:      input polynomial
pub fn poly_tomsg(msg: &mut[u8], a: Poly)
{
  let mut t : u32;

  for i in 0..KYBER_N/8 {
    msg[i] = 0;
    for j in 0..8 {
      t  = a.coeffs[8*i+j] as u32;

      t <<= 1;
      t = t.wrapping_add(1665);
      t = t.wrapping_mul(80635);
      t >>= 28;
      t &= 1;

      msg[i] |= (t << j) as u8;
    }
  }
}
