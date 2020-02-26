use crate::prelude::*;

use rand::prelude::*;

/// One piece of a packet.
#[derive(Debug, PartialEq)]
pub(crate) enum Chunk {
  /// A message to pass back to the [`Mesher`](../struct.Mesher.html)
  Message(Vec<u8>, Option<u8>),
  /// A path to send this packet along
  Transport(String),
  /// A chunk we couldn't decrypt (meant for someone else)
  Encrypted(Vec<u8>),
}

impl Chunk {
  /// Converts the Chunk into a series of bytes that represents it.
  /// Best considered a black box, so it can change freely.
  fn serialize(self) -> Vec<u8> {
    match self {
      Chunk::Message(mut m, reply_to) => {
        let mut b = vec![0];
        let reply_to = match reply_to {
          None => 0,
          Some(idx) => idx + 1,
        };
        b.push(reply_to);
        b.append(&mut m);
        b
      }
      Chunk::Transport(t) => {
        let mut b = vec![1];
        b.append(&mut t.into_bytes());
        b
      }
      Chunk::Encrypted(v) => v,
    }
  }

  /// Converts a series of bytes from [`Chunk::serialize`](#method.serialize) back to a Chunk, if possible.
  /// Best considered a black box, so it can change freely.
  fn deserialize(mut from: Vec<u8>) -> Result<Chunk, ()> {
    match from.get(0) {
      Some(0) => {
        let reply_to = match from[1] {
          0 => None,
          i => Some(i - 1),
        };
        Ok(Chunk::Message(from.drain(2..).collect(), reply_to))
      }
      Some(1) => Ok(Chunk::Transport(
        String::from_utf8(from.drain(1..).collect()).map_err(|_| ())?,
      )),
      _ => Err(()),
    }
  }

  /// Convert this chunk to a raw byte form, then encrypt those to the public key.
  ///
  /// Should be considered a black box, as the format may change in the future.
  /// It will, of course, always be decryptable (assuming the keys match) by [`Chunk::decrypt`](#method.decrypt)
  fn encrypt(self, target_key: &encrypt::PublicKey) -> Vec<u8> {
    encrypt::seal(&self.serialize(), target_key)
  }

  /// Same as [`Chunk::encrypt`](#method.encrypt), but will also sign with the sender key.
  fn encrypt_and_sign(self, target_key: &encrypt::PublicKey, sender_key: &sign::SecretKey) -> Vec<u8> {
    sign::sign(&encrypt::seal(&self.serialize(), target_key), sender_key)
  }

  /// Decrypt a chunk of bytes with all of our keys.
  /// Returns the chunk decrypted with the first key that worked.
  /// If none of them work, returns [`Chunk::Encrypted`](#variant.Encrypted).
  ///
  /// Expect the input format to this to be a black box.
  /// Give it things encrypted with [`Chunk::encrypt`](#method.encrypt).
  fn decrypt(bytes: Vec<u8>, keys: &[encrypt::SecretKey]) -> Chunk {
    for key in keys {
      if let Ok(dec) = encrypt::open(&bytes, key) {
        if let Ok(des) = Self::deserialize(dec) {
          return des;
        }
      }
    }
    Chunk::Encrypted(bytes)
  }

  /// Same as [`Chunk::decrypt`](#method.decrypt) but will check signatures against the list of signing keys.
  fn decrypt_signed(bytes: Vec<u8>, enc_keys: &[encrypt::SecretKey], sign_keys: &[sign::PublicKey]) -> Chunk {
    let veried = match sign_keys.iter().find_map(|k| sign::verify(&bytes, k).ok()) {
      Some(v) => v,
      None => return Chunk::Encrypted(bytes),
    };
    let decd = match enc_keys.iter().find_map(|k| encrypt::open(&veried, k).ok()) {
      Some(d) => d,
      None => return Chunk::Encrypted(bytes),
    };
    match Self::deserialize(decd) {
      Ok(d) => d,
      Err(_) => Chunk::Encrypted(bytes),
    }
  }
}

pub struct ReplyPathHandle<'packet>(u8, &'packet mut Packet);

impl<'packet> ReplyPathHandle<'packet> {
  /// Adds a message to the packet, for the node with the right skey to read.
  pub fn add_message<'handle>(&'handle mut self, data: &[u8], node_pkey: &encrypt::PublicKey, reply: Option<ReplyPathHandle<'handle>>) {
    self.1.add_instruction(Some(self.0), Chunk::Message(data.to_vec(), reply.map(|h| h.0)), node_pkey)
  }

  /// Adds a hop to the packet, so that when it reaches the node with the right skey, it'll get forwarded along the given path.
  pub fn add_hop(&mut self, path: String, node_pkey: &encrypt::PublicKey) {
    self.1.add_instruction(Some(self.0), Chunk::Transport(path), node_pkey)
  }

  /// Instructs anyone who gets a reply sent along this path how to reply.
  /// 
  /// This can "nest" infinitely, but that's only rarely useful.
  /// If you're replying back to whoever sent you the message, they constructed a path before, and can do it again.
  /// 
  /// Nest is in scare-quotes because, in the packet, there is no nesting -- it's treated like any other reply block.
  pub fn add_reply_path(&mut self) -> Option<ReplyPathHandle> {
    self.1.add_reply_path()
  }
}

/// Represents a packet to be sent out.
///
/// Note that each piece of the packet is associated with a key.
/// The keys don't have to be unique -- more than one piece can be associated with a single key.
/// For example, if a node is meant to both receive a message and transport the packet further, those two might be encrypted with the same key.
pub struct Packet {
  pub(crate) main_path: Vec<Vec<u8>>,
  pub(crate) reply_paths: Vec<Vec<Vec<u8>>>,
  pub(crate) signing_key: Option<sign::SecretKey>,
}

impl Packet {
  /// Creates a packet whose chunks won't be signed.
  pub fn unsigned() -> Packet {
    Packet {
      main_path: vec![],
      reply_paths: vec![],
      signing_key: None,
    }
  }

  /// Creates a packet whose chunks will be signed by the given key.
  pub fn signed(skey: sign::SecretKey) -> Packet {
    Packet {
      main_path: vec![],
      reply_paths: vec![],
      signing_key: Some(skey),
    }
  }

  fn add_instruction(&mut self, block: Option<u8>, instruct: Chunk, target_pkey: &encrypt::PublicKey) {
    let bytes = match &self.signing_key {
      Some(signing_key) => instruct.encrypt_and_sign(target_pkey, signing_key),
      None => instruct.encrypt(target_pkey),
    };
    match block {
      None => &mut self.main_path,
      Some(idx) => &mut self.reply_paths[idx as usize],
    }.push(bytes);
  }

  /// Adds a message to the packet, for the node with the right skey to read.
  pub fn add_message(&mut self, data: &[u8], node_pkey: &encrypt::PublicKey) {
    self.add_instruction(None, Chunk::Message(data.to_vec(), None), node_pkey)
  }

  /// Adds a hop to the packet, so that when it reaches the node with the right skey, it'll get forwarded along the given path.
  pub fn add_hop(&mut self, path: String, node_pkey: &encrypt::PublicKey) {
    self.add_instruction(None, Chunk::Transport(path), node_pkey)
  }

  pub fn add_reply_path(&mut self) -> Option<ReplyPathHandle> {
    if self.reply_paths.len() == u8::max_value() as usize {
      return None;
    }
    self.reply_paths.push(vec![]);
    Some(ReplyPathHandle(self.reply_paths.len() as u8 - 1, self))
  }

  /// Serializes the packet into a sendable format.
  pub(crate) fn into_bytes(mut self) -> fail::Result<Vec<u8>> {
    let mut rng = thread_rng();
    let mut paths = Vec::with_capacity(self.reply_paths.len() + 1);
    self.main_path.shuffle(&mut rng);
    paths.push(self.main_path);
    for mut path in self.reply_paths {
      path.shuffle(&mut rng);
      paths.push(path);
    }
    bincode::serialize(&paths).map_err(|e| fail::MesherFail::Other(Box::new(e)))
  }

  /// Given a packet and all of our secret keys, decrypt as many chunks as possible.
  ///
  /// No error is raised if no chunks could be decrypted; you just get a Vec entirely
  /// composed of [`Chunk::Encrypted`](enum.Chunk.html#variant.Encrypted).
  ///
  /// See [`Chunk::decrypt`](enum.Chunk.html#method.decrypt) for more information.
  pub(crate) fn from_bytes(packet: &[u8], keys: &[encrypt::SecretKey]) -> fail::Result<Vec<Chunk>> {
    bincode::deserialize::<Vec<Vec<u8>>>(packet)
      .map(|packet| packet.into_iter().map(|c| Chunk::decrypt(c, keys)).collect())
      .map_err(|_| fail::MesherFail::InvalidPacket)
  }

  /// Same as [`Packet::from_bytes`](#method.from_bytes) but only decrypts chunks signed with one of the valid keys.
  pub(crate) fn from_signed_bytes(
    packet: &[u8],
    keys: &[encrypt::SecretKey],
    sender_keys: &[sign::PublicKey],
  ) -> fail::Result<Vec<Chunk>> {
    bincode::deserialize::<Vec<Vec<u8>>>(packet)
      .map(|packet| {
        packet
          .into_iter()
          .map(|c| Chunk::decrypt_signed(c, keys, sender_keys))
          .collect()
      })
      .map_err(|_| fail::MesherFail::InvalidPacket)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // "borrowed" from https://doc.rust-lang.org/src/core/macros/mod.rs.html#264-271
  // TODO: Delete this once it's part of std
  macro_rules! matches {
    ($expression:expr, $( $pattern:pat )|+ $( if $guard: expr )?) => {
      match $expression {
        $( $pattern )|+ $( if $guard )? => true,
        _ => false
      }
    }
  }

  #[test]
  fn unsigned_serialized_deserializable() {
    let (pk1, sk1) = encrypt::gen_keypair();
    let (pk2, sk2) = encrypt::gen_keypair();

    let mut packet = Packet::unsigned();
    packet.add_hop("hello".to_owned(), &pk1);
    packet.add_message(&[1, 2, 3], &pk2);
    let packet = packet.into_bytes().expect("Failed to serialize packet");

    let dec1 = Packet::from_bytes(&packet, &[sk1]).expect("Failed to deserialize packets");
    assert!(dec1.contains(&Chunk::Transport("hello".to_owned())));
    assert!(dec1.iter().any(|c| matches!(c, Chunk::Encrypted(_))));

    let dec2 = Packet::from_bytes(&packet, &[sk2]).expect("Failed to deserialize packets");
    assert!(dec2.contains(&Chunk::Message(vec![1, 2, 3], None)));
    assert!(dec2.iter().any(|c| matches!(c, Chunk::Encrypted(_))));
  }

  #[test]
  fn signed_serialized_deserializable() {
    let (pks, sks) = sign::gen_keypair();
    let (pk1, sk1) = encrypt::gen_keypair();
    let (pk2, sk2) = encrypt::gen_keypair();

    let mut packet = Packet::signed(sks);
    packet.add_hop("hello".to_owned(), &pk1);
    packet.add_message(&[1, 2, 3], &pk2);
    let packet = packet.into_bytes().expect("Failed to serialize packet");

    let dec1 = Packet::from_signed_bytes(&packet, &[sk1], &[pks]).expect("Failed to deserialize packets");
    assert!(dec1.contains(&Chunk::Transport("hello".to_owned())));
    assert!(dec1.iter().any(|c| matches!(c, Chunk::Encrypted(_))));

    let dec2 = Packet::from_signed_bytes(&packet, &[sk2], &[pks]).expect("Failed to deserialize packets");
    assert!(dec2.contains(&Chunk::Message(vec![1, 2, 3], None)));
    assert!(dec2.iter().any(|c| matches!(c, Chunk::Encrypted(_))));
  }
}
