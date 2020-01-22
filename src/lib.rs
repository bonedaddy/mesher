#![warn(clippy::all)]

use std::collections::HashMap;
pub use {
  routing::Route,
  transports::{Transport, TransportFail},
  // x25519_dalek::PublicKey,
  // x25519_dalek::StaticSecret as SecretKey,
};

pub struct PublicKey(pub usize);
pub struct SecretKey(pub usize);

pub mod fail;
pub mod routing;
pub mod transports;

pub struct Message {
  contents: Vec<u8>,
  reply_route: Option<Route>,
}

impl Message {
  pub fn contents(&self) -> &[u8] {
    &self.contents
  }
}

pub struct Mesher {
  transports: HashMap<String, Box<dyn Transport>>,
}

impl Mesher {
  pub fn signed(_source_sigs: Vec<crate::PublicKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
    }
  }
  pub fn unsigned() -> Mesher {
    Mesher {
      transports: HashMap::new(),
    }
  }

  pub fn add_own_key(&mut self, _k: crate::SecretKey) {}
  pub fn add_sender_key(&mut self, _k: crate::PublicKey) {}

  pub fn add_transport<T: Transport + 'static>(
    &mut self,
    scheme: &str,
  ) -> Result<(), TransportFail> {
    self
      .transports
      .insert(scheme.to_owned(), Box::new(T::new(scheme)?));
    Ok(())
  }

  pub fn send(&mut self, _message: &[u8], _route: Route) -> fail::Result<()> {
    Ok(())
  }
  pub fn reply(&mut self, _message: &[u8], _to: Message) -> fail::Result<()> {
    Ok(())
  }

  pub fn recv(&mut self) -> fail::Result<Vec<Message>> {
    Ok(vec![])
  }
}
