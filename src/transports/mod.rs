pub trait Transport: Sized {
  type Fail: std::error::Error;
  fn new(prefix: &str) -> Result<Self, Self::Fail>;
  fn send(&mut self, blob: Vec<u8>) -> Result<(), Self::Fail>;
  fn recv(&mut self) -> Result<Vec<Vec<u8>>, Self::Fail>;
}

mod debug;
pub use debug::Debug;
