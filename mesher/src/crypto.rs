//! Mesher's wrapper around its crypto engine.
//!
//! While this is a nicer, easier-to-use wrapper around crypto primitives, using it still requires you to understand how public-key crypto works.
//! For example, if you don't know the security guarantees provided by encryption vs. signing, **do not use this wrapper**.

use ed25519_dalek as ed;
use rand::prelude::*;
use ring::aead::{self, BoundKey};
use std::{
  fmt,
  sync::{Arc, Mutex},
  time::SystemTime,
};
use x25519_dalek as x;

lazy_static! {
  static ref CURRENT: Arc<Mutex<u128>> = {
    let val = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(dur) => dur.as_nanos(),
      Err(_) => {
        let top = (thread_rng().next_u64() as u128) << 64;
        let bot = thread_rng().next_u64() as u128;
        top | bot
      }
    };
    Arc::new(Mutex::new(val))
  };
}

fn next_nonce() -> [u8; 12] {
  let val = {
    let mut d = CURRENT.lock().expect("Lock should never get poisoned");
    *d += 1;
    d
  };
  let bytes = val.to_le_bytes();
  let mut bytes_12 = [0; 12];
  bytes_12.copy_from_slice(&bytes[0..12]);
  bytes_12
}

struct SingleNonce {
  nonce_bytes: [u8; 12],
  to_serve: bool,
}

impl From<[u8; 12]> for SingleNonce {
  fn from(n: [u8; 12]) -> SingleNonce {
    SingleNonce {
      nonce_bytes: n,
      to_serve: true,
    }
  }
}

impl aead::NonceSequence for SingleNonce {
  fn advance(&mut self) -> Result<aead::Nonce, ring::error::Unspecified> {
    if self.to_serve {
      self.to_serve = false;
      Ok(aead::Nonce::assume_unique_for_key(self.nonce_bytes))
    } else {
      Err(ring::error::Unspecified)
    }
  }
}

/// The public half of the keypair.
///
/// It's used to *en*crypt things and check signatures.
/// It can be automatically derived from the secret key with [`SecretKey::pkey`](struct.SecretKey.html#method.pkey).
#[derive(Clone)]
pub struct PublicKey {
  edkey: ed::PublicKey,
  xkey: x::PublicKey,
}

impl PublicKey {
  /// Recreates a key from material gotten from [`PublicKey::material`](#method.material).
  ///
  /// # WARNING
  ///
  /// This method is dangerous if not used properly!
  /// Even if the raw bytes passed are generated sufficiently randomly, they may not be a valid or secure key.
  /// Either make completely certain you fully understand the underlying crypto math being used, or just use [`SecretKey::generate`](struct.SecretKey.html#method.generate) to produce new keys, and [`SecretKey::pkey`](struct.SecretKey.html#method.pkey) to get the public key.
  pub fn load(material: [u8; 64]) -> Result<PublicKey, ()> {
    let edkey = ed::PublicKey::from_bytes(&material[..32]).map_err(|_| ())?;
    let mut x_bytes = [0; 32];
    x_bytes.copy_from_slice(&material[32..]);
    let xkey = x::PublicKey::from(x_bytes);
    Ok(PublicKey { edkey, xkey })
  }

  /// Gets the key material out of this key, so it can be stored.
  ///
  /// Ideally, avoid using this method.
  /// However, in some applications (e.g. servers with published public keys) it's extremely useful or even necessary to keep using the same key, so if you need to "export" a `SecretKey`, this will allow you to.
  /// You **must** know what you're doing, though!
  ///
  /// You don't need to store the public key if you have the secret key because it can be trivially recreated from the private key.
  pub fn material(&self) -> [u8; 64] {
    let mut out = [0; 64];
    out[..32].copy_from_slice(self.edkey.as_bytes());
    out[32..].copy_from_slice(self.xkey.as_bytes());
    out
  }

  /// Encrypts a bunch of data with this public key.
  /// Only the associated secret key can decrypt it.
  ///
  /// The return value's format should be considered, by and large, a black box.
  /// Just pass it to [`SecretKey::decrypt`](struct.SecretKey.html#method.decrypt) to decrypt the message.
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  pub(crate) fn encrypt(&self, data: &[u8]) -> Vec<u8> {
    // variable names from README.md § Operation description
    let rs = x::StaticSecret::new(&mut thread_rng());
    let rp = x::PublicKey::from(&rs);
    let s = rs.diffie_hellman(&self.xkey);

    let nonce = next_nonce();
    let ukey = aead::UnboundKey::new(&aead::AES_256_GCM, s.as_bytes()).expect("Size should have matched");
    let mut key = aead::SealingKey::new(ukey, SingleNonce::from(nonce));

    let mut aad = Vec::with_capacity(32 + 12);
    aad.extend_from_slice(rp.as_bytes());
    aad.extend_from_slice(&nonce);

    let mut cipher = data.to_vec();
    key
      .seal_in_place_append_tag(aead::Aad::from(&aad), &mut cipher)
      .expect("Should never cause an error");
    let mut out = Vec::with_capacity(32 /* rp */ + 12 /* nonce */ + cipher.len());
    out.extend_from_slice(rp.as_bytes());
    out.extend_from_slice(&nonce);
    out.append(&mut cipher);
    out
  }

  /// Checks that the message was signed by this PublicKey.
  ///
  /// The input's format should be considered, by and large, a black box.
  /// Just use what's returned by [`SecretKey::sign`](struct.SecretKey.html#method.sign).
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns a Result rather than a bool to help prevent unverified messages from being used accidentally.
  pub(crate) fn verify(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    if ciphertext.len() < 64 {
      return Err(());
    }
    let (sig, plaintext) = ciphertext.split_at(64);
    let sig = ed::Signature::from_bytes(sig).map_err(|_| ())?;
    if self.edkey.verify_strict(&plaintext, &sig).is_ok() {
      Ok(plaintext.to_vec())
    } else {
      Err(())
    }
  }
}

impl fmt::Debug for PublicKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", &self.material()[..])
  }
}

impl PartialEq for PublicKey {
  fn eq(&self, rhs: &PublicKey) -> bool {
    self.material()[..] == rhs.material()[..]
  }
}

/// The secret half of the keypair.
///
/// It's used to *de*crypt things and create signatures.
///
/// The public half can be derived with [`SecretKey::pkey`](#method.pkey).
pub struct SecretKey {
  edkey: ed::SecretKey,
  xkey: x::StaticSecret,
}

impl SecretKey {
  /// Securely generate a new secret key.
  ///
  /// This function makes its best effort to be cryptographically secure by relying on the OS's CSRNG.
  /// However, in certain (rare) circumstances, the OS's CSRNG may not actually be cryptographically secure, e.g. when not enough entropy is available.
  ///
  /// In those cases, or when you want to load a stored key, use [`SecretKey::load`](#method.load).
  ///
  /// To get the public key of the freshly generated key, use [`SecretKey::pkey`](#method.pkey).
  pub fn generate() -> SecretKey {
    let edkey = ed::SecretKey::generate(&mut thread_rng());
    let xkey = x::StaticSecret::new(&mut thread_rng());
    SecretKey { edkey, xkey }
  }

  /// Recreates a key from material gotten from [`SecretKey::material`](#method.material).
  ///
  /// To get the public key of the freshly loaded key, use [`SecretKey::pkey`](#method.pkey).
  ///
  /// # WARNING
  ///
  /// This method is dangerous if not used properly!
  /// Even if the raw bytes passed are generated sufficiently randomly, they may not be a secure key.
  /// Either make completely certain you fully understand the underlying crypto math being used, or just use [`SecretKey::generate`](#method.generate) to produce new keys.
  pub fn load(material: [u8; 64]) -> Result<SecretKey, ()> {
    let edkey = ed::SecretKey::from_bytes(&material[..32]).map_err(|_| ())?;
    let mut x_bytes = [0; 32];
    x_bytes.copy_from_slice(&material[32..]);
    let xkey = x::StaticSecret::from(x_bytes);
    Ok(SecretKey { edkey, xkey })
  }

  /// Gets the key material out of this key, so it can be stored.
  ///
  /// Ideally, avoid using this method.
  /// However, in some applications (e.g. servers with published public keys) it's extremely useful or even necessary to keep using the same key, so if you need to "export" a `SecretKey`, this will allow you to.
  /// You **must** know what you're doing, though!
  ///
  /// You don't need to store the public key if you have the secret key because it can be trivially recreated from the private key.
  pub fn material(&self) -> [u8; 64] {
    let mut out = [0; 64];
    out[..32].copy_from_slice(self.edkey.as_bytes());
    out[32..].copy_from_slice(&self.xkey.to_bytes());
    out
  }

  /// Derive the public half of the keypair based on the secret key.
  pub fn pkey(&self) -> PublicKey {
    let edkey = ed::PublicKey::from(&self.edkey);
    let xkey = x::PublicKey::from(&self.xkey);
    PublicKey { edkey, xkey }
  }

  /// Does the same thing as [`SecretKey::pkey`](#method.pkey) but returns a tuple of the two keys, which is ergonomically easier in some usages.
  pub fn pair(self) -> (SecretKey, PublicKey) {
    let pk = self.pkey();
    (self, pk)
  }

  /// Decrypts a bunch of data that was encrypted with the associated public key.
  /// If it doesn't seem to actually be targeting this secret key, returns Err(())
  ///
  /// The input's format should be considered, by and large, a black box.
  /// Just use what's returned by [`PublicKey::encrypt`](struct.PublicKey.html#method.encrypt).
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  pub(crate) fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    if ciphertext.len() < (32 + 12) {
      return Err(());
    }

    let (aad, ciphertext) = ciphertext.split_at(32 + 12);
    let (rp_slice, nonce_slice) = aad.split_at(32);

    let mut rp_bytes = [0u8; 32];
    rp_bytes.copy_from_slice(rp_slice);
    let rp = x::PublicKey::from(rp_bytes);
    let s = self.xkey.diffie_hellman(&rp);

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(nonce_slice);

    let ukey = aead::UnboundKey::new(&aead::AES_256_GCM, s.as_bytes()).expect("Size should have matched");
    let mut key = aead::OpeningKey::new(ukey, SingleNonce::from(nonce_bytes));

    let mut ciphertext = ciphertext.to_vec();
    match key.open_in_place(aead::Aad::from(aad), &mut ciphertext) {
      Ok(plain) => Ok(plain.to_vec()),
      Err(_) => Err(()),
    }
  }

  /// Signs this message with the given secret key.
  ///
  /// The return value's format should be considered, by and large, a black box.
  /// Just pass it to [`PublicKey::verify`](struct.PublicKey.html#method.verify) to check the signature.
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  pub(crate) fn sign(&self, data: &[u8]) -> Vec<u8> {
    let esk = ed::ExpandedSecretKey::from(&self.edkey);
    let epk = ed::PublicKey::from(&esk);
    let sig = esk.sign(data, &epk);
    let mut out = Vec::with_capacity(data.len() + ed::SIGNATURE_LENGTH);
    out.extend_from_slice(&sig.to_bytes());
    out.extend_from_slice(data);
    out
  }
}

impl Clone for SecretKey {
  fn clone(&self) -> SecretKey {
    let edkey = ed::SecretKey::from_bytes(self.edkey.as_bytes()).expect("Pulled from valid");
    let xkey = self.xkey.clone();
    SecretKey { edkey, xkey }
  }
}

impl fmt::Debug for SecretKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", &self.material()[..])
  }
}

impl PartialEq for SecretKey {
  fn eq(&self, rhs: &SecretKey) -> bool {
    self.material()[..] == rhs.material()[..]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encryption_decryptable() {
    let (sk, pk) = SecretKey::generate().pair();

    let data = vec![1, 2, 3, 4];
    let encd = pk.encrypt(&data);
    let decd = sk.decrypt(&encd);

    assert_eq!(Ok(vec![1, 2, 3, 4]), decd);
  }

  #[test]
  fn signature_verifiable() {
    let (sk, pk) = SecretKey::generate().pair();

    let data = vec![1, 2, 3, 4];
    let signed = sk.sign(&data);
    let veried = pk.verify(&signed);

    assert_eq!(Ok(vec![1, 2, 3, 4]), veried);
  }

  #[test]
  fn encrypt_and_sign() {
    let (sks, pks) = SecretKey::generate().pair();
    let (skr, pkr) = SecretKey::generate().pair();

    let data = vec![1, 2, 3, 4];
    let encd = pkr.encrypt(&data);
    let signed = sks.sign(&encd);
    let out = pks.verify(&signed).and_then(|v| skr.decrypt(&v));
    assert_eq!(Ok(data), out);
  }

  #[test]
  fn enc_nondeterministic() {
    let pk = SecretKey::generate().pkey();
    let data = &[1, 2, 3, 4];
    let out1 = pk.encrypt(data);
    let out2 = pk.encrypt(data);
    assert_ne!(out1, out2);
  }

  #[test]
  fn skey_save_load() {
    let (sk1, pk) = SecretKey::generate().pair();
    let sk_bytes = sk1.material();
    let sk2 = SecretKey::load(sk_bytes).expect("Pulled from valid");

    let data = vec![1, 2, 3, 4];
    let enc = pk.encrypt(&data);
    let res1 = sk1.decrypt(&enc).expect("Should be valid");
    let res2 = sk2.decrypt(&enc).expect("Should be valid");

    assert_eq!(data, res1);
    assert_eq!(data, res2);
  }

  #[test]
  fn pkey_save_load() {
    let (sk, pk1) = SecretKey::generate().pair();
    let pk_bytes = pk1.material();
    let pk2 = PublicKey::load(pk_bytes).expect("Pulled from valid");

    let data = vec![1, 2, 3, 4];
    let enc1 = pk1.encrypt(&data);
    let enc2 = pk2.encrypt(&data);

    // encryption should be nondeterministic
    assert_ne!(enc1, enc2);

    let res1 = sk.decrypt(&enc1).expect("Should be valid");
    let res2 = sk.decrypt(&enc2).expect("Should be valid");

    // but still decrypt to the same thing
    assert_eq!(data, res1);
    assert_eq!(data, res2);
  }
}
