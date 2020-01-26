#![warn(clippy::all)]

#[cfg(feature = "debug")]
#[macro_use]
extern crate lazy_static;

mod crypto;
mod fail;
mod packet;
pub mod transports;

use {
  crypto::{PublicKey, SecretKey},
  transports::{Transport, TransportFail},
  rand::prelude::*,
  std::collections::HashMap,
};

#[derive(Debug, Clone)]
pub struct Route {
  target: crate::PublicKey,
  first_hop: String,
  transports: Vec<(String, crate::PublicKey)>,
  // TODO: Replies
}

impl Route {
  pub fn to(target_key: &crate::PublicKey, first_hop: &str) -> Route {
    Route {
      target: target_key.clone(),
      first_hop: first_hop.to_owned(),
      transports: Vec::new(),
    }
  }
  pub fn with_transport(mut self, node_key: &crate::PublicKey, transport: &str) -> Route {
    self
      .transports
      .push((transport.to_owned(), node_key.clone()));
    self
  }
}

#[derive(Debug)]
pub struct Message {
  contents: Vec<u8>,
}

impl Message {
  pub fn contents(&self) -> &[u8] {
    &self.contents
  }
}

pub struct Mesher {
  transports: HashMap<String, Box<dyn Transport>>,
  own_skeys: Vec<crate::SecretKey>,
  own_pkeys: Vec<crate::PublicKey>,
  rng: rand::rngs::ThreadRng,
}

impl Mesher {
  pub fn signed(own_skeys: Vec<crate::SecretKey>, _source_sigs: Vec<crate::PublicKey>) -> Mesher {
    // TODO: outgoing packet signature setup
    Mesher::unsigned(own_skeys)
  }
  pub fn unsigned(own_skeys: Vec<crate::SecretKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
      own_pkeys: own_skeys.iter().map(SecretKey::pkey).collect(),
      own_skeys,
      rng: ThreadRng::default(),
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

  #[allow(clippy::borrowed_box)]
  fn get_transport_for_path(
    &mut self,
    path: &str,
  ) -> Result<&mut Box<dyn Transport>, TransportFail> {
    let scheme = path
      .splitn(2, ':')
      .next()
      .ok_or(transports::TransportFail::InvalidURL(
        "no colon-delimited scheme segment",
      ))?
      .to_owned();
    self
      .transports
      .get_mut(&scheme)
      .ok_or(transports::TransportFail::UnregisteredScheme(scheme))
  }

  pub fn listen_on(&mut self, path: &str) -> Result<(), TransportFail> {
    self.get_transport_for_path(path)?.listen(path.to_owned())
  }

  fn random_key(&mut self) -> fail::Result<PublicKey> {
    self
      .own_pkeys
      .choose(&mut self.rng)
      .map(Clone::clone)
      .ok_or(fail::Fail::NoKeys)
  }

  fn process_packet(&mut self, pkt: Vec<u8>) -> fail::Result<Vec<Message>> {
    let dis = packet::disassemble(&pkt, &self.own_skeys)?;
    let mut messages = vec![];
    for piece in dis {
      match piece {
        packet::Chunk::Message(m) => messages.push(Message { contents: m }),
        packet::Chunk::Transport(to) => self.bounce(&pkt, &to)?,
        packet::Chunk::Encrypted(_) => (/* piece not meant for us */),
      }
    }
    Ok(messages)
  }

  pub fn send(&mut self, message: &[u8], route: Route) -> fail::Result<()> {
    let assembled = packet::assemble(message, route, self.random_key()?)?;
    self.process_packet(assembled)?;
    Ok(())
  }

  fn bounce(&mut self, packet: &[u8], path: &str) -> fail::Result<()> {
    let transport = self.get_transport_for_path(path)?;
    transport.send(path.to_owned(), packet.to_vec())?;
    Ok(())
  }

  pub fn recv(&mut self) -> fail::Result<Vec<Message>> {
    // don't focus too much on how I got this...
    let mut packets = vec![];
    for (_, transport) in self.transports.iter_mut() {
      packets.append(&mut transport.receive()?);
    }
    let mut messages = vec![];
    for p in packets {
      messages.append(&mut self.process_packet(p)?);
    }
    Ok(messages)
  }
}

pub mod prelude {
  pub use crate::{
    crypto::{PublicKey, SecretKey},
    transports::{Transport, TransportFail},
    Mesher, Route,
  };
}
