//! Contains all the relevant bits and pieces for meshers themselves.

use crate::prelude::*;
use std::{
  collections::HashMap,
  sync::Arc,
};

/// Represents a single message received by a mesher.
#[derive(Debug, PartialEq)]
pub struct Message {
  contents: Vec<u8>,
  pub(crate) reply_path: Option<Arc<Vec<Vec<u8>>>>,
}

impl Message {
  /// Get the contents of the message.
  pub fn contents(&self) -> &[u8] {
    &self.contents
  }

  /// Get the contents of the message, discarding the `Message` struct.
  pub fn into_contents(self) -> Vec<u8> {
    self.contents
  }

  /// Whether or not this message was sent with a reply path for it to follow.
  pub fn has_reply_path(&self) -> bool {
    self.reply_path.is_some()
  }
}

/// The control interface for a single mesher.
///
/// One important thing to note is that the Mesher struct **only** stores keys during runtime.
/// It does not manage them in any other way, e.g. keeping them securely on-disk, transmitting them securely to the computer, etc.
/// (However, you could well use messages passed through mesher to handle some of it.)
pub struct Mesher {
  transports: HashMap<String, Box<dyn Transport>>,
  own_skeys: Vec<encrypt::SecretKey>,
  sender_pkeys: Vec<sign::PublicKey>,
}

impl Mesher {
  /// Creates a mesher which expects incoming messages to be signed with one of the given keys.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned meshers' packets.
  /// Signed meshers will expect their incoming packets to have signatures; unsigned meshers won't.
  /// If a signing mesher receives an unsigned packet or vice versa, it'll be a no-op.
  pub fn signed(own_skeys: Vec<encrypt::SecretKey>, sender_pkeys: Vec<sign::PublicKey>) -> Mesher {
    assert!(
      !sender_pkeys.is_empty(),
      "Provide sender keys. If you don't want any, use Mesher::unsigned instead."
    );

    Mesher {
      transports: HashMap::new(),
      own_skeys,
      sender_pkeys,
    }
  }

  /// Creates a mesher which doesn't sign its outgoing messages.
  /// The keys are used when receiving messages, to decrypt the ones meant for it.
  ///
  /// Note that there are no (explicit) markers to differentiate between signed and unsigned meshers.
  /// Signed meshers will expect their incoming packets to have signatures; unsigned meshers won't.
  /// If a signing mesher receives an unsigned packet or vice versa, it'll be a no-op.
  pub fn unsigned(own_skeys: Vec<encrypt::SecretKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
      own_skeys,
      sender_pkeys: vec![],
    }
  }

  /// Does the massaging necessary to get the transport based on the scheme in the path.
  /// Will return the appropriate errors if any of it fails.
  #[allow(clippy::borrowed_box)] // because we can't easily massage &mut Box<T> into &mut T, apparently
  fn get_transport_for_path(&mut self, path: &str) -> fail::Result<&mut Box<dyn Transport>> {
    let scheme = path
      .splitn(2, ':')
      .next()
      .ok_or_else(|| fail::MesherFail::InvalidURL("no colon-delimited scheme segment".to_string()))?
      .to_owned();
    self
      .transports
      .get_mut(&scheme)
      .ok_or(fail::MesherFail::UnregisteredScheme(scheme))
  }

  /// Does everything you'd expect when mesher receives a packet:
  ///
  /// - Attempts to decrypt every line in the packet
  /// - Forwards the packet as dictated by it
  /// - Returns any messages contained in it
  ///
  /// It will try to use _all_ of the secret keys associated with the mesher to decrypt the packet.
  fn process_packet(&mut self, pkt: Vec<u8>) -> fail::Result<Vec<Message>> {
    let dis = if self.sender_pkeys.is_empty() {
      Packet::deserialize(&pkt, &self.own_skeys)?
    } else {
      Packet::deserialize_signed(&pkt, &self.own_skeys, &self.sender_pkeys)?
    };
    let mut messages = vec![];
    for piece in dis {
      match piece {
        crate::packet::Chunk::Message(m, r) => messages.push(Message { contents: m, reply_path: r }),
        crate::packet::Chunk::Transport(to) => self.send_data(&pkt, &to)?,
      }
    }
    Ok(messages)
  }

  // Sends the given bytes along the given path, getting the appropriate transport.
  fn send_data(&mut self, packet: &[u8], path: &str) -> fail::Result<()> {
    self.get_transport_for_path(path)?.send(path.to_owned(), packet.to_vec())
  }

  /// Adds a transport to the mesher, for it to send and receive data through.
  /// The scheme is passed to the transport exactly as-is.
  /// If an initialization error occurs in the transport, nothing is added to the internal scheme mapping.
  pub fn add_transport<T: Transport + 'static>(&mut self, scheme: &str) -> fail::Result<()> {
    self.transports.insert(scheme.to_owned(), Box::new(T::new(scheme)?));
    Ok(())
  }

  /// Has the mesher listen on the given path for messages.
  /// This determines the transport to connect to based on the scheme, then just tells it to listen.
  /// The exact behavior depends on the transport, but will generally involve either setting up some listener, or adding it to a list of internal paths to poll.
  pub fn listen_on(&mut self, path: &str) -> fail::Result<()> {
    self.get_transport_for_path(path)?.listen(path.to_owned())
  }

  /// Sends a packet out.
  /// 
  /// Note that while the outgoing packet is processed like any incoming one, any messages destined for this mesher are ignored.
  pub fn launch(&mut self, packet: Packet) -> fail::Result<()> {
    self.process_packet(packet.serialize()?).map(|_| ())
  }

  /// Gets pending messages from all of the transports along all of the paths they've been told to use.
  pub fn receive(&mut self) -> fail::Result<Vec<Message>> {
    if self.own_skeys.is_empty() {
      return Err(fail::MesherFail::NoKeys);
    }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_mesher_fails_receive() {
    let mut empty = Mesher::unsigned(vec![]);

    match empty.receive() {
      Err(fail::MesherFail::NoKeys) => (),
      _ => unreachable!(),
    }
  }

  #[test]
  #[should_panic(expected = "Provide sender keys. If you don't want any, use Mesher::unsigned instead.")]
  fn signed_mesher_empty_keys_fails() {
    let _empty = Mesher::signed(vec![], vec![]);
  }
}
