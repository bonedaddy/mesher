#[non_exhaustive]
#[derive(Debug)]
pub enum TransportFail {
  // the packet we received isn't formatted validly
  InvalidPacket,

  // the URL is syntactically invalid
  InvalidURL(&'static str),
  // the scheme hasn't been registered with the Mesher
  UnregisteredScheme(String),

  // could not establish a connection along the transport
  ConnectionFailure(String),

  // an arbitary other error
  Other(Box<dyn std::error::Error>),
}

pub trait Transport {
  fn new(scheme: &str) -> Result<Self, TransportFail>
  where
    Self: Sized;
  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), TransportFail>;
  fn listen(&mut self, path: String) -> Result<(), TransportFail>;
  fn receive(&mut self) -> Result<Vec<Vec<u8>>, TransportFail>;
}

pub mod standard;

#[cfg(any(feature = "debug"))]
pub mod debug;
