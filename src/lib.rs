#![warn(clippy::all)]

pub mod fail;
mod packet;
pub mod transports;

pub use transports::{Transport, TransportFail};
use {rand::prelude::*, std::collections::HashMap};

#[derive(Debug, Clone)]
pub struct PublicKey(pub usize);
#[derive(Debug, Clone)]
pub struct SecretKey(pub usize);

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
    Mesher {
      transports: HashMap::new(),
      own_pkeys: own_skeys.iter().map(|sk| PublicKey(sk.0)).collect(),
      own_skeys,
      rng: ThreadRng::default(),
    }
  }
  pub fn unsigned(own_skeys: Vec<crate::SecretKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
      own_pkeys: own_skeys.iter().map(|sk| PublicKey(sk.0)).collect(),
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

  fn random_key(&mut self) -> fail::Result<PublicKey> {
    self
      .own_pkeys
      .choose(&mut self.rng)
      .map(Clone::clone)
      .ok_or(fail::Fail::NoKeys)
  }

  fn process_packet(&mut self, pkt: Vec<u8>) -> fail::Result<Vec<Message>> {
    let dis = packet::disassemble(&pkt, &self.own_skeys);
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
    println!("Sending {:?} along {:?}", message, route);
    let assembled = packet::assemble(message, route, self.random_key()?);
    println!("packet is: {:?}", assembled);
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
      4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 109, 101, 115, 104, 0, 1, 2, 3, 23, 0, 0, 0,
      0, 0, 0, 0, 112, 104, 118, 107, 4, 103, 104, 101, 120, 106, 61, 118, 104, 113, 103, 105, 108,
      117, 118, 119, 107, 114, 115, 20, 0, 0, 0, 0, 0, 0, 0, 110, 102, 116, 105, 2, 101, 102, 99,
      118, 104, 59, 116, 102, 111, 101, 113, 98, 117, 105, 50, 20, 0, 0, 0, 0, 0, 0, 0, 111, 103,
      117, 106, 3, 102, 103, 100, 119, 105, 60, 117, 103, 112, 102, 114, 99, 118, 106, 52,
    ]];
    let mut messages = vec![];
    for p in packets {
      messages.append(&mut self.process_packet(p)?);
    }
    Ok(messages)
  }
}
