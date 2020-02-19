//! Rust's wrapper around its crypto engine.
//! **WARNING**: Currently intentionally very broken!
//! This crypto is NOT SECURE and should not be used in production!
//! It's intentionally easy to break so that if I need to, while debugging, I can.
//! 
//! While this is a nicer, easier-to-use wrapper around crypto primitives, using it still requires you to understand how public-key crypto works.
//! For example, if you don't know the security guarantees provided by encryption vs. signing, **do not use this wrapper**.

// TODO: Replace with real crypto

/// Some magic bytes to indicate if stuff is ours.
/// Only used in this bad crypto impl; the real one will use authenticated encryption.
const MAGIC: &[u8] = &[0x6d, 0x65, 0x73, 0x68]; // "mesh" in ASCII

/// The public half of the keypair.
///
/// It's used to *en*crypt things and check signatures.
/// It can be automatically derived from the secret key with [`SecretKey::pkey`][1].
///
///  [1]: struct.SecretKey.html#method.pkey
#[derive(Debug, Clone)]
pub struct PublicKey(u8);
impl PublicKey {
  /// **Insecurely** generate a public key, deterministically, based off a name.
  ///
  /// Unsafe because this is a **terrible** way to generate keys!
  /// It's only good for demos -- e.g. the various examples, where proper key management would add too much complexity.
  #[allow(clippy::missing_safety_doc)]
  pub unsafe fn of(name: &str) -> PublicKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    PublicKey(sum)
  }

  /// Encrypts a bunch of data with this public key.
  /// Only the associated secret key can decrypt it.
  ///
  /// The return value's format should be considered, by and large, a black box.
  /// Just pass it to [`SecretKey::decrypt`][1] to decrypt the message.
  /// This ensures that the crypto can be upgraded without requiring any other code to change
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  /// 
  ///  [1]: struct.SecretKey.html#method.decrypt
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
  /// Just use what's returned by [`SecretKey::sign`][1]
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns a Result rather than a bool to help prevent unverified messages from being used accidentally.
  /// 
  ///  [1]: struct.SecretKey.html#method.sign
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
/// The public half can be derived with [`SecretKey::pkey`][1].
///
///  [1]: #method.pkey
#[derive(Debug, Clone)]
pub struct SecretKey(u8);
impl SecretKey {
  /// **Insecurely** generate a secret key, deterministically, based off a name.
  ///
  /// Unsafe because this is a **terrible** way to generate keys!
  /// It's only good for demos -- e.g. the various examples.
  #[allow(clippy::missing_safety_doc)]
  pub unsafe fn of(name: &str) -> SecretKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    SecretKey(sum)
  }

  /// Decrypts a bunch of data that was encrypted with the associated public key.
  /// If it doesn't seem to actually be targeting this secret key, returns Err(())
  ///
  /// The input's format should be considered, by and large, a black box.
  /// Just use what's returned by [`PublicKey::encrypt`][1].
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  ///
  ///  [1]: struct.PublicKey.html#method.encrypt
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
  /// Just pass it to [`PublicKey::verify`][1] to check the signature.
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// This returns an Option to help ensure that the message can't be accidentally taken without ensuring a valid signature.
  /// 
  ///  [1]: struct.PublicKey.html#method.verify
  pub(crate) fn sign(&self, data: &[u8]) -> Vec<u8> {
    let sig = data.iter().fold(0u8, |a, i| a.wrapping_add(*i)).wrapping_add(self.0);
    let mut res = data.to_vec();
    res.push(sig);
    res
  }

  /// Derive the public half of the keypair based on the secret key.
  pub fn pkey(&self) -> PublicKey {
    PublicKey(self.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encryption_decryptable() {
    let pk = unsafe { PublicKey::of("crypt") };
    let sk = unsafe { SecretKey::of("crypt") };

    let encd = pk.encrypt(&[1, 2, 3, 4]);
    let decd = sk.decrypt(&encd);

    assert_eq!(Ok(vec![1, 2, 3, 4]), decd);
  }

  #[test]
  fn signature_verifiable() {
    let pk = unsafe { PublicKey::of("crypt") };
    let sk = unsafe { SecretKey::of("crypt") };

    let signed = sk.sign(&[1, 2, 3, 4]);
    let veried = pk.verify(&signed);

    assert_eq!(Ok(vec![1, 2, 3, 4]), veried);
  }
}
