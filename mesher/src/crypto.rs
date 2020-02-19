//! Rust's wrapper around its crypto engine.
//! **WARNING**: Currently intentionally very broken!
//! This crypto is NOT SECURE and should not be used in production!
//! It's intentionally easy to break so that if I need to, while debugging, I can.

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
  /// This ensures that the crypto can be upgraded without requiring any other code to change
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

  /// Encrypts a bunch of data and signs it with the given secret key.
  /// Only the associated secret key can decrypt it, but anyone can check that it's signed with the corresponding public key.
  ///
  /// The return value's format should be considered, by and large, a black box.
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  pub(crate) fn encrypt_and_sign(&self, data: &[u8], signer: &SecretKey) -> Vec<u8> {
    let mut encd = self.encrypt(data);
    let signature = encd.iter().fold(0u8, |a, i| a.wrapping_add(*i)).wrapping_add(signer.0);
    encd.push(signature);
    encd
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

  /// Decrypts a bunch of data that was encrypted with the associated public key, checking that it's signed by the public key.
  /// If it doesn't seem to actually be targeting this secret key, or the signature is invalid, returns Err(())
  ///
  /// The input's format should be considered, by and large, a black box.
  /// Just use what's returned by [`PublicKey::encrypt`][1].
  /// This ensures that the crypto can be upgraded without requiring any other code to change.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned ciphertexts.
  /// The meshers will know based on how they're initialized.
  ///
  ///  [1]: struct.PublicKey.html#method.encrypt
  pub(crate) fn decrypt_signed(&self, ciphertext: &[u8], signer: &PublicKey) -> Result<Vec<u8>, ()> {
    let (ciphertext, signature) = ciphertext.split_at(ciphertext.len() - 1);
    let signature = signature.get(0).ok_or(())?;
    if signature.wrapping_sub(signer.0) == ciphertext.iter().fold(0u8, |a, i| a.wrapping_add(*i)) {
      self.decrypt(ciphertext)
    } else {
      Err(())
    }
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
  fn signed_encryption_decryptable_checked() {
    let send_pk = unsafe { PublicKey::of("send") };
    let send_sk = unsafe { SecretKey::of("send") };
    let recv_pk = unsafe { PublicKey::of("recv") };
    let recv_sk = unsafe { SecretKey::of("recv") };

    let encd = recv_pk.encrypt_and_sign(&[1, 2, 3, 4], &send_sk);
    let decd = recv_sk.decrypt_signed(&encd, &send_pk);

    assert_eq!(Ok(vec![1, 2, 3, 4]), decd);
  }
}
