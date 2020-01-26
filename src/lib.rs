#![warn(clippy::all)]

#[cfg(feature = "debug")]
#[macro_use]
extern crate lazy_static;

mod crypto;
pub mod fail;
mod packet;
pub mod transports;

pub use {
  crypto::{PublicKey, SecretKey},
  transports::{Transport, TransportFail},
};
use {rand::prelude::*, std::collections::HashMap};

#[derive(Debug, Clone)]
pub struct Route {
  target: crate::PublicKey,
  first_hop: String,
  transports: Vec<(String, crate::PublicKey)>,
  // TODO: Replies
}

impl Route {
  pub fn to(target_key: crate::PublicKey, first_hop: &str) -> Route {
    println!("Creating route to {:?}", target_key);
    Route {
      target: target_key,
      first_hop: first_hop.to_owned(),
      transports: Vec::new(),
    }
  }
  pub fn with_transport(mut self, node_key: &crate::PublicKey, transport: &str) -> Route {
    println!("Adding transport {} for node {:?}", transport, node_key);
    self
      .transports
      .push((transport.to_owned(), node_key.clone()));
    self
  }
}

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

  pub fn listen_on(&mut self, path: &str) -> Result<(), TransportFail> {
    let scheme = path
      .splitn(2, ':')
      .next()
      .ok_or(transports::TransportFail::InvalidURL(
        "no colon-delimited scheme segment",
      ))?
      .to_owned();
    let transport = self
      .transports
      .get_mut(&scheme)
      .ok_or(transports::TransportFail::UnregisteredScheme(scheme))?;
    transport.listen(path.to_owned())
  }

  fn random_key(&mut self) -> fail::Result<PublicKey> {
    self
      .own_pkeys
      .choose(&mut self.rng)
      .map(Clone::clone)
      .ok_or(fail::Fail::NoKeys)
  }

  fn process_packet(&mut self, pkt: Vec<u8>) -> fail::Result<Vec<Message>> {
    let dis = packet::disassemble(&pkt, &self.own_skeys);
    println!("Disassembled packet: {:?}", dis);
    let mut messages = vec![];
    for piece in dis {
      match piece {
        Ok(packet::Chunk::Message(m)) => messages.push(Message { contents: m }),
        Ok(packet::Chunk::Transport(to)) => self.bounce(&pkt, &to)?,
        Err(_) => (/* piece not meant for us */),
      }
    }
    Ok(messages)
  }

  pub fn send(&mut self, message: &[u8], route: Route) -> fail::Result<()> {
    println!("Sending message {:?} along {:?}", message, route);
    let assembled = packet::assemble(message, route, self.random_key()?);
    println!("Packet being sent is: {:?}", assembled);
    self.process_packet(assembled)?;
    Ok(())
  }
  pub fn reply(&mut self, _message: &[u8], _to: Message) -> fail::Result<()> {
    Err(fail::Fail::NotYetImplemented("Message replies"))
  }

  fn bounce(&mut self, packet: &[u8], transport: &str) -> fail::Result<()> {
    println!("Would send {:?} along {:?}", packet, transport);
    Ok(())
  }

  pub fn recv(&mut self) -> fail::Result<Vec<Message>> {
    // don't focus too much on how I got this...
    let packets = vec![vec![
      4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 244, 236, 250, 239, 135, 136, 137, 138, 23,
      0, 0, 0, 0, 0, 0, 0, 49, 41, 55, 44, 197, 40, 41, 38, 57, 43, 254, 55, 41, 50, 40, 42, 45,
      54, 55, 56, 44, 51, 52, 20, 0, 0, 0, 0, 0, 0, 0, 12, 4, 18, 7, 160, 3, 4, 1, 20, 6, 217, 18,
      4, 13, 3, 15, 0, 19, 7, 208, 20, 0, 0, 0, 0, 0, 0, 0, 13, 5, 19, 8, 161, 4, 5, 2, 21, 7, 218,
      19, 5, 14, 4, 16, 1, 20, 8, 210,
    ]];
    let mut messages = vec![];
    for p in packets {
      messages.append(&mut self.process_packet(p)?);
    }
    Ok(messages)
  }
}
