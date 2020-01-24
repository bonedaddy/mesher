#![warn(clippy::all)]

pub mod fail;
pub mod transports;
mod packet;

use std::collections::HashMap;
pub use {
  transports::{Transport, TransportFail},
  // x25519_dalek::PublicKey,
  // x25519_dalek::StaticSecret as SecretKey,
};

#[derive(Debug)]
pub struct Route {
  target: crate::PublicKey,
  first_hop: String,
  transports: Vec<(String, crate::PublicKey)>,
  reply: Option<Box<Route>>,
}

impl Route {
  pub fn to(target_key: crate::PublicKey, first_hop: &str) -> Route {
    println!("Creating route to {:?}", target_key);
    Route {
      target: target_key,
      first_hop: first_hop.to_owned(),
      transports: Vec::new(),
      reply: None,
    }
  }
  pub fn with_transport(mut self, node_key: &crate::PublicKey, transport: &str) -> Route {
    println!("Adding transport {} for node {:?}", transport, node_key);
    self
      .transports
      .push((transport.to_owned(), node_key.clone()));
    self
  }
  pub fn reply_to(mut self, path: Route) -> Route {
    println!("Directing replies along {:?}", path);
    self.reply = Some(Box::new(path));
    self
  }
}

#[derive(Debug, Clone)]
pub struct PublicKey(pub usize);
#[derive(Debug, Clone)]
pub struct SecretKey(pub usize);

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
  pub fn signed(_own_skeys: Vec<crate::SecretKey>, _source_sigs: Vec<crate::PublicKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
    }
  }
  pub fn unsigned(_own_skeys: Vec<crate::SecretKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
    }
  }

  pub fn add_transport<T: Transport + 'static>(
    &mut self,
    scheme: &str,
  ) -> Result<(), TransportFail> {
    self
      .transports
      .insert(scheme.to_owned(), Box::new(T::new(scheme)?));
    Ok(())
  }

  pub fn send(&mut self, message: &[u8], route: Route) -> fail::Result<()> {
    println!("Sending {:?} along {:?}", message, route);
    Ok(())
  }
  pub fn reply(&mut self, _message: &[u8], _to: Message) -> fail::Result<()> {
    Ok(())
  }

  pub fn recv(&mut self) -> fail::Result<Vec<Message>> {
    Ok(vec![])
  }
}
