// TODO: Replace with real crypto


#[derive(Debug, Clone)]
pub struct PublicKey(u8, String);
impl PublicKey {
  // Unsafe because this is a /terrible/ way to generate keys
  #[allow(clippy::missing_safety_doc)]
  pub unsafe fn of(name: &str) -> PublicKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    PublicKey(sum, name.to_owned())
  }

  pub(crate) fn encrypt(&self, data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b.wrapping_add(self.0)).collect()
  }
}

#[derive(Debug, Clone)]
pub struct SecretKey(u8, String);
impl SecretKey {
  // Unsafe because this is a /terrible/ way to generate keys
  #[allow(clippy::missing_safety_doc)]
  pub unsafe fn of(name: &str) -> SecretKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    SecretKey(sum, name.to_owned())
  }

  pub(crate) fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    Ok(ciphertext.iter().map(|b| b.wrapping_sub(self.0)).collect())
  }

  pub fn pkey(&self) -> PublicKey {
    PublicKey(self.0, self.1.clone())
  }
}
