use crate::prelude::*;

use std::sync::Arc;

use rand::prelude::*;

/// A chunk being added into a packet
#[derive(Debug, PartialEq)]
enum InputChunk {
  /// A message to pass back to the [`Mesher`](../struct.Mesher.html)
  Message(Vec<u8>, Option<u8>),
  /// A path to send this packet along
  Transport(String),
}

impl InputChunk {
  /// Converts the Chunk into a series of bytes that represents it.
  /// Best considered a black box, so it can change freely.
  fn serialize(self) -> Vec<u8> {
    match self {
      InputChunk::Message(mut m, reply_to) => {
        let mut b = vec![0];
        let reply_to = match reply_to {
          None => 0,
          Some(idx) => idx + 1,
        };
        b.push(reply_to);
        b.append(&mut m);
        b
      }
      InputChunk::Transport(t) => {
        let mut b = vec![1];
        b.append(&mut t.into_bytes());
        b
      }
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
}

/// One piece of a packet being parsed on receipt.
#[derive(Debug, PartialEq)]
pub(crate) enum Chunk {
  /// A message to pass back to the [`Mesher`](../struct.Mesher.html)
  Message(Vec<u8>, Option<Arc<Vec<Vec<u8>>>>),
  /// A path to send this packet along
  Transport(String),
  /// A chunk we couldn't decrypt (meant for someone else)
  Encrypted(Vec<u8>),
}

impl Chunk {
  /// Converts a series of bytes from [`Chunk::serialize`](#method.serialize) back to a Chunk, if possible.
  /// Best considered a black box, so it can change freely.
  fn deserialize(mut from: Vec<u8>, replies: &[Arc<Vec<Vec<u8>>>]) -> Result<Chunk, ()> {
    match from.get(0) {
      Some(0) => {
        let reply = match from[1] {
          0 => None,
          i => Some(replies.get(i as usize - 1).expect("Should be valid index").clone()),
        };
        Ok(Chunk::Message(from.drain(2..).collect(), reply))
      }
      Some(1) => Ok(Chunk::Transport(
        String::from_utf8(from.drain(1..).collect()).map_err(|_| ())?,
      )),
      _ => Err(()),
    }
  }

  /// Decrypt a chunk of bytes with all of our keys.
  /// Returns the chunk decrypted with the first key that worked.
  /// If none of them work, returns [`Chunk::Encrypted`](#variant.Encrypted).
  ///
  /// Expect the input format to this to be a black box.
  /// Give it things encrypted with [`Chunk::encrypt`](#method.encrypt).
  fn decrypt(bytes: Vec<u8>, replies: &[Arc<Vec<Vec<u8>>>], keys: &[encrypt::SecretKey]) -> Chunk {
    for key in keys {
      if let Ok(dec) = encrypt::open(&bytes, key) {
        if let Ok(des) = Self::deserialize(dec, replies) {
          return des;
        }
      }
    }
    Chunk::Encrypted(bytes)
  }

  /// Same as [`Chunk::decrypt`](#method.decrypt) but will check signatures against the list of signing keys.
  fn decrypt_signed(bytes: Vec<u8>, replies: &[Arc<Vec<Vec<u8>>>], enc_keys: &[encrypt::SecretKey], sign_keys: &[sign::PublicKey]) -> Chunk {
    let veried = match sign_keys.iter().find_map(|k| sign::verify(&bytes, k).ok()) {
      Some(v) => v,
      None => return Chunk::Encrypted(bytes),
    };
    let decd = match enc_keys.iter().find_map(|k| encrypt::open(&veried, k).ok()) {
      Some(d) => d,
      None => return Chunk::Encrypted(bytes),
    };
    match Self::deserialize(decd, replies) {
      Ok(d) => d,
      Err(_) => Chunk::Encrypted(bytes),
    }
  }
}

pub struct ReplyPathHandle<'packet>(u8, &'packet mut Packet);

impl<'packet> ReplyPathHandle<'packet> {
  /// Adds a message to the packet, for the node with the right skey to read.
  pub fn add_message<'handle>(&'handle mut self, data: &[u8], node_pkey: &encrypt::PublicKey, reply: Option<ReplyPathHandle<'handle>>) {
    self.1.add_instruction(Some(self.0), InputChunk::Message(data.to_vec(), reply.map(|h| h.0)), node_pkey)
  }

  /// Adds a hop to the packet, so that when it reaches the node with the right skey, it'll get forwarded along the given path.
  pub fn add_hop(&mut self, path: String, node_pkey: &encrypt::PublicKey) {
    self.1.add_instruction(Some(self.0), InputChunk::Transport(path), node_pkey)
  }

  /// Adds a message to the packet, for the node with the right skey to read, and to reply along the given path.
  pub fn use_for_message(&mut self, data: &[u8], node_pkey: &encrypt::PublicKey) {
    self.1.add_instruction(None, InputChunk::Message(data.to_vec(), Some(self.0)), node_pkey)
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

  fn add_instruction(&mut self, block: Option<u8>, instruct: InputChunk, target_pkey: &encrypt::PublicKey) {
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
    self.add_instruction(None, InputChunk::Message(data.to_vec(), None), node_pkey)
  }

  /// Adds a hop to the packet, so that when it reaches the node with the right skey, it'll get forwarded along the given path.
  pub fn add_hop(&mut self, path: String, node_pkey: &encrypt::PublicKey) {
    self.add_instruction(None, InputChunk::Transport(path), node_pkey)
  }

  /// Starts creating a reply path.
  pub fn add_reply_path(&mut self) -> Option<ReplyPathHandle> {
    if self.reply_paths.len() == u8::max_value() as usize {
      return None;
    }
    self.reply_paths.push(vec![]);
    Some(ReplyPathHandle(self.reply_paths.len() as u8 - 1, self))
  }

  /// Serializes the packet into a sendable format.
  pub(crate) fn serialize(mut self) -> fail::Result<Vec<u8>> {
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
  pub(crate) fn deserialize(packet: &[u8], keys: &[encrypt::SecretKey]) -> fail::Result<Vec<Chunk>> {
    let blocks = bincode::deserialize::<Vec<Vec<Vec<u8>>>>(packet);
    let mut blocks = match blocks {
      Ok(blocks) if !blocks.is_empty() => blocks,
      _ => return Err(fail::MesherFail::InvalidPacket),
    };
    let replies: Vec<_> = blocks.split_off(1).into_iter().map(Arc::new).collect();
    let main = blocks.pop().expect("Already validated length before");
    let main = main
      .into_iter()
      .map(|c| Chunk::decrypt(c, &replies, keys))
      .collect();
    Ok(main)
  }

  /// Same as [`Packet::from_bytes`](#method.from_bytes) but only decrypts chunks signed with one of the valid keys.
  pub(crate) fn deserialize_signed(
    packet: &[u8],
    keys: &[encrypt::SecretKey],
    sender_keys: &[sign::PublicKey],
  ) -> fail::Result<Vec<Chunk>> {
    let blocks = bincode::deserialize::<Vec<Vec<Vec<u8>>>>(packet);
    let mut blocks = match blocks {
      Ok(blocks) if !blocks.is_empty() => blocks,
      _ => return Err(fail::MesherFail::InvalidPacket),
    };
    let replies: Vec<_> = blocks.split_off(1).into_iter().map(Arc::new).collect();
    let main = blocks.pop().expect("Already validated length before");
    let main = main
      .into_iter()
      .map(|c| Chunk::decrypt_signed(c, &replies, keys, sender_keys))
      .collect();
    Ok(main)
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
    let packet = packet.serialize().expect("Failed to serialize packet");

    let dec1 = Packet::deserialize(&packet, &[sk1]).expect("Failed to deserialize packets");
    assert!(dec1.contains(&Chunk::Transport("hello".to_owned())));
    assert!(dec1.iter().any(|c| matches!(c, Chunk::Encrypted(_))));

    let dec2 = Packet::deserialize(&packet, &[sk2]).expect("Failed to deserialize packets");
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
    let packet = packet.serialize().expect("Failed to serialize packet");

    let dec1 = Packet::deserialize_signed(&packet, &[sk1], &[pks]).expect("Failed to deserialize packets");
    assert!(dec1.contains(&Chunk::Transport("hello".to_owned())));
    assert!(dec1.iter().any(|c| matches!(c, Chunk::Encrypted(_))));

    let dec2 = Packet::deserialize_signed(&packet, &[sk2], &[pks]).expect("Failed to deserialize packets");
    assert!(dec2.contains(&Chunk::Message(vec![1, 2, 3], None)));
    assert!(dec2.iter().any(|c| matches!(c, Chunk::Encrypted(_))));
  }

  #[test]
  fn all_functions_compile() {
    // These functions have kinda fucky lifetime stuff, so let's just have a "test" to ensure they compile when used as expected...

    let (pk, _) = encrypt::gen_keypair();
    let mut packet = Packet::unsigned();

    packet.add_hop("foo1".to_owned(), &pk);
    packet.add_message(&[1], &pk);
    
    let mut rh1 = packet.add_reply_path().expect("Failed to add reply handle 1");
    rh1.add_hop("foo2".to_owned(), &pk);
    rh1.use_for_message(&[2], &pk);

    let mut rh2 = packet.add_reply_path().expect("Failed to add reply handle 2");
    rh2.add_hop("foo4".to_owned(), &pk);
    rh2.use_for_message(&[4], &pk);
  }

  #[test]
  fn replies_match() {
    use std::collections::HashMap;

    let (pk, sk) = encrypt::gen_keypair();
    let (tpk, _) = encrypt::gen_keypair();
    let bytes = {
      let mut packet = Packet::unsigned();

      packet.add_hop("foo1".to_owned(), &tpk);
      packet.add_message(&[1], &pk);
      
      let mut rh1 = packet.add_reply_path().expect("Failed to add reply handle 1");
      rh1.add_hop("foo2".to_owned(), &tpk);
      rh1.use_for_message(&[2], &pk);
      rh1.use_for_message(&[3], &pk);

      let mut rh2 = packet.add_reply_path().expect("Failed to add reply handle 2");
      rh2.add_hop("foo4".to_owned(), &tpk);
      rh2.use_for_message(&[4], &pk);
      rh2.use_for_message(&[5], &pk);
      
      packet.serialize().expect("Failed to serialize packet")
    };
    
    let deser = Packet::deserialize(&bytes, &[sk]).expect("Failed to deserialize");
    let mut messages = HashMap::new();
    for chunk in deser {
      if let Chunk::Message(data, rep) = chunk {
        messages.insert(data[0], rep);
      }
    }

    // first message has no reply path
    assert_eq!(messages[&1], None);

    // 2 and 3 should have the same reply path; same for 4 and 5
    assert_eq!(messages[&2], messages[&3]);
    assert_eq!(messages[&4], messages[&5]);

    // neither 2/3 nor 4/5 should be None
    assert_ne!(messages[&2], None);
    assert_ne!(messages[&4], None);
    // and 2/3 and 4/5 should be different
    assert_ne!(messages[&3], messages[&4]);
  }

  #[test]
  fn signed_replies_match() {
    use std::collections::HashMap;

    let (rpk, rsk) = encrypt::gen_keypair();
    let (spk, ssk) = sign::gen_keypair();
    let (tpk, _) = encrypt::gen_keypair();
    let bytes = {
      let mut packet = Packet::signed(ssk);

      packet.add_hop("foo1".to_owned(), &tpk);
      packet.add_message(&[1], &rpk);
      
      let mut rh1 = packet.add_reply_path().expect("Failed to add reply handle 1");
      rh1.add_hop("foo2".to_owned(), &tpk);
      rh1.use_for_message(&[2], &rpk);
      rh1.use_for_message(&[3], &rpk);

      let mut rh2 = packet.add_reply_path().expect("Failed to add reply handle 2");
      rh2.add_hop("foo4".to_owned(), &tpk);
      rh2.use_for_message(&[4], &rpk);
      rh2.use_for_message(&[5], &rpk);
      
      packet.serialize().expect("Failed to serialize packet")
    };
    
    let deser = Packet::deserialize_signed(&bytes, &[rsk], &[spk]).expect("Failed to deserialize");
    let mut messages = HashMap::new();
    for chunk in deser {
      if let Chunk::Message(data, rep) = chunk {
        messages.insert(data[0], rep);
      }
    }

    // first message has no reply path
    assert_eq!(messages[&1], None);

    // 2 and 3 should have the same reply path; same for 4 and 5
    assert_eq!(messages[&2], messages[&3]);
    assert_eq!(messages[&4], messages[&5]);

    // neither 2/3 nor 4/5 should be None
    assert_ne!(messages[&2], None);
    assert_ne!(messages[&4], None);
    // and 2/3 and 4/5 should be different
    assert_ne!(messages[&3], messages[&4]);
  }
}
