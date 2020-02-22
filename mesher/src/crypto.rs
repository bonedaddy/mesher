//! Rust's wrapper around its crypto engine.
//! **WARNING**: Currently intentionally very broken!
//! This crypto is NOT SECURE and should not be used in production!
//! It's intentionally easy to break so that if I need to, while debugging, I can.
//! 
//! While this is a nicer, easier-to-use wrapper around crypto primitives, using it still requires you to understand how public-key crypto works.
//! For example, if you don't know the security guarantees provided by encryption vs. signing, **do not use this wrapper**.

// TODO: Replace with real crypto

use rand::prelude::*;

/// Some magic bytes to indicate if stuff is ours.
/// Only used in this bad crypto impl; the real one will use authenticated encryption.
const MAGIC: &[u8] = &[0x6d, 0x65, 0x73, 0x68]; // "mesh" in ASCII

/// The public half of the keypair.
///
/// It's used to *en*crypt things and check signatures.
/// It can be automatically derived from the secret key with [`SecretKey::pkey`](struct.SecretKey.html#method.pkey).
#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey(u8);
impl PublicKey {
  /// Recreates a key from material gotten from [`PublicKey::material`](#method.material).
  /// 
  /// # WARNING
  /// 
  /// This method is dangerous if not used properly!
  /// Even if the raw bytes passed are generated sufficiently randomly, they may not be a secure key.
  /// Either make completely certain you fully understand the underlying crypto math being used, or just use [`SecretKey::generate`](struct.SecretKey.html#method.generate) to produce new keys, and [`SecretKey::pkey`](struct.SecretKey.html#method.pkey) to get the public key.
  pub fn load(material: [u8; 32]) -> PublicKey {
    PublicKey(material.iter().fold(0u8, |a, i| a.wrapping_add(*i)))
  }

  /// Gets the key material out of this key, so it can be stored.
  /// 
  /// Ideally, avoid using this method.
  /// However, in some applications (e.g. servers with published public keys) it's extremely useful or even necessary to keep using the same key, so if you need to "export" a `SecretKey`, this will allow you to.
  /// You **must** know what you're doing, though!
  /// 
  /// You don't need to store the public key if you have the secret key because it can be trivially recreated from the private key.
  pub fn material(self) -> [u8; 32] {
    let mut a = [0; 32];
    a[0] = self.0;
    a
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
    MAGIC
      .iter()
      .chain(data.iter())
      .map(|b| b.wrapping_add(self.0))
      .collect()
  }

  /// Checks that the message was signed by this PublicKey.
  /// 
  /// The input's format should be considered, by and large, a black box.
  /// Just use what's returned by [`SecretKey::sign`](struct.SecretKey.html#method.sign).
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns a Result rather than a bool to help prevent unverified messages from being used accidentally.
  pub(crate) fn verify(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    let mut ciphertext = ciphertext.to_vec();
    let sig = ciphertext.pop().ok_or(())?;
    if sig.wrapping_sub(self.0) == ciphertext.iter().fold(0u8, |a, i| a.wrapping_add(*i)) {
      Ok(ciphertext)
    } else {
      Err(())
    }
  }
}

/// The secret half of the keypair.
///
/// It's used to *de*crypt things and create signatures.
///
/// The public half can be derived with [`SecretKey::pkey`](#method.pkey).
#[derive(Debug, Clone, PartialEq)]
pub struct SecretKey(u8);
impl SecretKey {
  /// **Insecurely** generate a secret key, deterministically, based off a name.
  ///
  /// # Safety
  /// 
  /// This function can **never** be cryptographically secure, and thus will never be "safe" to use.
  /// The only safe, secure way to generate keys is with a source of cryptographically secure randomness.
  /// To generate a key safely, use [`SecretKey::generate`](#method.generate).
  /// 
  /// This method only exists because, while debugging or writing tests, broken (deterministic) keygen can be useful, and in those cases, safety isn't a concern.
  /// This particular method also preserves the name in the key data, for the same reason.
  /// 
  /// It's a drop-in, deterministic replacement for `SecretKey::generate`, so that you can swap it in and out easily for debugging.
  pub unsafe fn of(name: &str) -> SecretKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    SecretKey(sum)
  }

  /// Securely generate a new secret key.
  /// 
  /// This function makes its best effort to be cryptographically secure by relying on the OS's CSRNG.
  /// However, in certain (rare) circumstances, the OS's CSRNG may not actually be cryptographically secure, e.g. when not enough entropy is available.
  /// 
  /// In those cases, or when you want to load a stored key, use [`SecretKey::load`](#method.load).
  /// 
  /// To get the public key of the freshly generated key, use [`SecretKey::pkey`](#method.pkey).
  pub fn generate() -> SecretKey {
    SecretKey(thread_rng().next_u32() as u8)
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
    SecretKey(material.iter().fold(0u8, |a, i| a.wrapping_add(*i)))
  }

  /// Gets the key material out of this key, so it can be stored.
  /// 
  /// Ideally, avoid using this method.
  /// However, in some applications (e.g. servers with published public keys) it's extremely useful or even necessary to keep using the same key, so if you need to "export" a `SecretKey`, this will allow you to.
  /// You **must** know what you're doing, though!
  /// 
  /// You don't need to store the public key if you have the secret key because it can be trivially recreated from the private key.
  pub fn material(self) -> [u8; 32] {
    let mut a = [0; 32];
    a[0] = self.0;
    a
  }

  /// Derive the public half of the keypair based on the secret key.
  pub fn pkey(&self) -> PublicKey {
    PublicKey(self.0)
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
    let mut dec: Vec<_> = ciphertext.iter().map(|b| b.wrapping_sub(self.0)).collect();
    if &dec[0..4] != MAGIC {
      Err(())
    } else {
      Ok(dec.split_off(4))
    }
  }

  /// Signs this message with the given secret key.
  /// 
  /// The return value's format should be considered, by and large, a black box.
  /// Just pass it to [`PublicKey::verify`](struct.PublicKey.html#method.verify) to check the signature.
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns an Option to help ensure that the message can't be accidentally taken without ensuring a valid signature.
  pub(crate) fn sign(&self, data: &[u8]) -> Vec<u8> {
    let sig = data.iter().fold(0u8, |a, i| a.wrapping_add(*i)).wrapping_add(self.0);
    let mut res = data.to_vec();
    res.push(sig);
    res
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
  fn of_deterministic() {
    let sk1 = unsafe { SecretKey::of("some string") };
    let sk2 = unsafe { SecretKey::of("some string") };
    assert_eq!(sk1, sk2);
  }

  #[test]
  #[should_panic] // TODO: Fix the crypto and remove this
  fn enc_nondeterministic() {
    let pk = SecretKey::generate().pkey();
    let data = &[1, 2, 3, 4];
    let out1 = pk.encrypt(data);
    let out2 = pk.encrypt(data);
    assert_ne!(out1, out2);
  }
}
