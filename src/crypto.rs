#[derive(Debug)]
pub struct PublicKey(u8, String);
impl PublicKey {
  pub fn of(name: &str) -> PublicKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    PublicKey(sum, name.to_owned())
  }

  pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b + self.0).collect()
  }
}

#[derive(Debug)]
pub struct SecretKey(u8, String);
impl SecretKey {
  pub fn of(name: &str) -> SecretKey {
    let sum = name.as_bytes().iter().fold(0u8, |a, i| a.wrapping_add(*i));
    SecretKey(sum, name.to_owned())
  }

  pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    Ok(ciphertext.iter().map(|b| b - self.0).collect())
  }

  pub fn pkey(&self) -> PublicKey {
    PublicKey(self.0, self.1.clone())
  }
}
