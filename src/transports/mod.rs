pub trait Transport {
  type Fail: std::error::Error;
  fn new(prefix: &str) -> Self;
  fn send(&mut self, blob: Vec<u8>) -> Result<(), Self::Fail>;
  fn recv(&mut self) -> Result<Vec<Vec<u8>>, Self::Fail>;
}
