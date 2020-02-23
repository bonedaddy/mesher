//! Rust's wrapper around its crypto engine.
//! **WARNING**: Currently intentionally very broken!
//! This crypto is NOT SECURE and should not be used in production!
//! It's intentionally easy to break so that if I need to, while debugging, I can.
//! 
//! While this is a nicer, easier-to-use wrapper around crypto primitives, using it still requires you to understand how public-key crypto works.
//! For example, if you don't know the security guarantees provided by encryption vs. signing, **do not use this wrapper**.

use std::{sync::{Arc, Mutex}, fmt};
use rand::prelude::*;
use ed25519_dalek as ed;
use x25519_dalek as x;
use ring::aead::{self, BoundKey};

#[derive(Clone)]
struct NonceGen {
  current: Arc<Mutex<u128>>,
}

impl NonceGen {
  fn new() -> NonceGen {
    let val = match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
      Ok(dur) => dur.as_nanos(),
      Err(_) => {
        let top = (thread_rng().next_u64() as u128) << 64;
        let bot = thread_rng().next_u64() as u128;
        top | bot
      }
    };
    NonceGen {
      current: Arc::new(Mutex::new(val))
    }
  }
}

impl aead::NonceSequence for NonceGen {
  fn advance(&mut self) -> Result<aead::Nonce, ring::error::Unspecified> {
    let val = {
      let mut d = self.current.lock().map_err(|_| ring::error::Unspecified)?;
      *d += 1;
      d
    };
    let bytes = val.to_le_bytes();
    let mut bytes_12 = [0; 12];
    bytes_12.copy_from_slice(&bytes[0..12]);
    let nonce = aead::Nonce::assume_unique_for_key(bytes_12);
    Ok(nonce)
  }
}

lazy_static! {
  static ref NONCE_GEN: NonceGen = NonceGen::new();
}

/// The public half of the keypair.
///
/// It's used to *en*crypt things and check signatures.
/// It can be automatically derived from the secret key with [`SecretKey::pkey`](struct.SecretKey.html#method.pkey).
#[derive(Clone)]
pub struct PublicKey(x::PublicKey);
impl PublicKey {
  /// Recreates a key from material gotten from [`PublicKey::material`](#method.material).
  /// 
  /// # WARNING
  /// 
  /// This method is dangerous if not used properly!
  /// Even if the raw bytes passed are generated sufficiently randomly, they may not be a secure key.
  /// Either make completely certain you fully understand the underlying crypto math being used, or just use [`SecretKey::generate`](struct.SecretKey.html#method.generate) to produce new keys, and [`SecretKey::pkey`](struct.SecretKey.html#method.pkey) to get the public key.
  pub fn load(material: [u8; 32]) -> PublicKey {
    PublicKey(x::PublicKey::from(material))
  }

  /// Gets the key material out of this key, so it can be stored.
  /// 
  /// Ideally, avoid using this method.
  /// However, in some applications (e.g. servers with published public keys) it's extremely useful or even necessary to keep using the same key, so if you need to "export" a `SecretKey`, this will allow you to.
  /// You **must** know what you're doing, though!
  /// 
  /// You don't need to store the public key if you have the secret key because it can be trivially recreated from the private key.
  pub fn material(self) -> [u8; 32] {
    *self.0.as_bytes()
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
    let s = rs.diffie_hellman(&self.0);

    let mut key = aead::SealingKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, s.as_bytes()).expect("Size should have matched"), (*NONCE_GEN).clone());

    let mut cipher = data.to_vec();
    // TODO: Put Rp_bytes in AAD
    key.seal_in_place_append_tag(aead::Aad::empty(), &mut cipher).expect("what error could this possibly be??");
    let mut rp_bytes = rp.as_bytes().to_vec();
    rp_bytes.append(&mut cipher);
    rp_bytes
  }

  /// Checks that the message was signed by this PublicKey.
  /// 
  /// The input's format should be considered, by and large, a black box.
  /// Just use what's returned by [`SecretKey::sign`](struct.SecretKey.html#method.sign).
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns a Result rather than a bool to help prevent unverified messages from being used accidentally.
  pub(crate) fn verify(&self, _ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    unimplemented!()
  }
}

impl PartialEq for PublicKey {
  fn eq(&self, rhs: &PublicKey) -> bool {
    self.0.as_bytes() == rhs.0.as_bytes()
  }
}

impl fmt::Debug for PublicKey {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    let hex = self.0.as_bytes().iter().fold(String::with_capacity(64), |a, i| a + &format!("{:02X}", i));
    write!(fmt, "PublicKey({})", hex)
  }
}

/// The secret half of the keypair.
///
/// It's used to *de*crypt things and create signatures.
///
/// The public half can be derived with [`SecretKey::pkey`](#method.pkey).
#[derive(Clone)]
pub struct SecretKey(x::StaticSecret);
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
    let edgen = ed::SecretKey::generate(&mut thread_rng()).to_bytes();
    let xkey = x::StaticSecret::from(edgen);
    SecretKey(xkey)
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
  pub fn load(material: [u8; 32]) -> SecretKey {
    SecretKey(x::StaticSecret::from(material))
  }

  /// Gets the key material out of this key, so it can be stored.
  /// 
  /// Ideally, avoid using this method.
  /// However, in some applications (e.g. servers with published public keys) it's extremely useful or even necessary to keep using the same key, so if you need to "export" a `SecretKey`, this will allow you to.
  /// You **must** know what you're doing, though!
  /// 
  /// You don't need to store the public key if you have the secret key because it can be trivially recreated from the private key.
  pub fn material(self) -> [u8; 32] {
    self.0.to_bytes()
  }

  /// Derive the public half of the keypair based on the secret key.
  pub fn pkey(&self) -> PublicKey {
    PublicKey(x::PublicKey::from(&self.0))
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
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  pub(crate) fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    if ciphertext.len() < 32 {
      return Err(());
    }

    let (rp_byte_slice, ciphertext) = ciphertext.split_at(32);
    let mut rp_bytes = [0u8; 32];
    rp_bytes.copy_from_slice(rp_byte_slice);
    let rp = x::PublicKey::from(rp_bytes);
    let s = self.0.diffie_hellman(&rp);

    let mut key = aead::OpeningKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, s.as_bytes()).expect("Size should have matched"), NONCE_GEN.clone());

    let mut ciphertext = ciphertext.to_vec();
    key.open_in_place(aead::Aad::empty(), &mut ciphertext).map_err(|_| ())?;

    Ok(ciphertext) // freshly decrypted
  }

  /// Signs this message with the given secret key.
  /// 
  /// The return value's format should be considered, by and large, a black box.
  /// Just pass it to [`PublicKey::verify`](struct.PublicKey.html#method.verify) to check the signature.
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns an Option to help ensure that the message can't be accidentally taken without ensuring a valid signature.
  pub(crate) fn sign(&self, _data: &[u8]) -> Vec<u8> {
    unimplemented!()
  }
}

impl PartialEq for SecretKey {
  fn eq(&self, rhs: &SecretKey) -> bool {
    self.0.to_bytes() == rhs.0.to_bytes()
  }
}

impl fmt::Debug for SecretKey {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    let hex = self.0.to_bytes().iter().fold(String::with_capacity(64), |a, i| a + &format!("{:02X}", i));
    write!(fmt, "PublicKey({})", hex)
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
  fn sk_save_load() {
    let sk = SecretKey::generate();
    let saved = sk.clone().material();
    let loaded = SecretKey::load(saved);
    assert_eq!(sk, loaded);
  }

  #[test]
  fn pk_save_load() {
    let pk = SecretKey::generate().pkey();
    let saved = pk.clone().material();
    let loaded = PublicKey::load(saved);
    assert_eq!(pk, loaded);
  }
}
