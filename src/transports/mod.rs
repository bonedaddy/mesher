#[non_exhaustive]
#[derive(Debug)]
pub enum TransportFail {
  UnsupportedScheme(&'static str),

  ArgumentError(String),
  ConnectionFailure(String),

  Other(Box<dyn std::error::Error>),
}

pub trait Transport {
  fn new(scheme: &str) -> Result<Self, TransportFail>
  where
    Self: Sized;
  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), TransportFail>;
  fn recv(&mut self) -> Result<Vec<Vec<u8>>, TransportFail>;
}

mod debug;
pub use debug::Debug;
