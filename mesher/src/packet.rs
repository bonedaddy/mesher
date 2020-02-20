use crate::prelude::*;

/// One piece of a packet.
#[derive(Debug, PartialEq)]
pub(crate) enum Chunk {
  /// A message to pass back to the [`Mesher`](../struct.Mesher.html)
  Message(Vec<u8>),
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
      Chunk::Message(mut m) => {
        let mut b = vec![0];
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
      Some(0) => Ok(Chunk::Message(from.drain(1..).collect())),
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
  fn encrypt(self, target_key: PublicKey) -> Vec<u8> {
    target_key.encrypt(&self.serialize())
  }

  /// Same as [`Chunk::encrypt`](#method.encrypt), but will also sign with the sender key.
  fn encrypt_and_sign(self, target_key: PublicKey, sender_key: &SecretKey) -> Vec<u8> {
    sender_key.sign(&target_key.encrypt(&self.serialize()))
  }

  /// Decrypt a chunk of bytes with all of our keys.
  /// Returns the chunk decrypted with the first key that worked.
  /// If none of them work, returns [`Chunk::Encrypted`](#variant.Encrypted).
  ///
  /// Expect the input format to this to be a black box.
  /// Give it things encrypted with [`Chunk::encrypt`](#method.encrypt).
  fn decrypt(bytes: Vec<u8>, keys: &[SecretKey]) -> Chunk {
    for key in keys {
      if let Ok(dec) = key.decrypt(&bytes) {
        if let Ok(des) = Self::deserialize(dec) {
          return des;
        }
      }
    }
    Chunk::Encrypted(bytes)
  }

  /// Same as [`Chunk::decrypt`](#method.decrypt) but will check signatures against the list of signing keys.
  fn decrypt_signed(bytes: Vec<u8>, enc_keys: &[SecretKey], sign_keys: &[PublicKey]) -> Chunk {
    let veried = match sign_keys.iter().find_map(|k| k.verify(&bytes).ok()) {
      Some(v) => v,
      None => return Chunk::Encrypted(bytes),
    };
    let decd = match enc_keys.iter().find_map(|k| k.decrypt(&veried).ok()) {
      Some(d) => d,
      None => return Chunk::Encrypted(bytes),
    };
    match Self::deserialize(decd) {
      Ok(d) => d,
      Err(_) => Chunk::Encrypted(bytes),
    }
  }
}

/// Represents a packet to be sent out.
///
/// Note that each piece of the packet is associated with a key.
/// The keys don't have to be unique -- more than one piece can be associated with a single key.
/// For example, if a node is meant to both receive a message and transport the packet further, those two might be encrypted with the same key.
pub struct Packet {
  chunks: Vec<(Chunk, PublicKey)>,
  signing_key: Option<SecretKey>,
}

impl Packet {
  /// Creates a packet whose chunks won't be signed.
  pub fn unsigned() -> Packet {
    Packet {
      chunks: vec![],
      signing_key: None,
    }
  }

  /// Creates a packet whose chunks will be signed by the given key.
  pub fn signed(skey: SecretKey) -> Packet {
    Packet {
      chunks: vec![],
      signing_key: Some(skey),
    }
  }

  /// Adds a message to the packet, for the node with the right skey to read.
  pub fn add_message(mut self, data: &[u8], target_pkey: &PublicKey) -> Packet {
    self.chunks.push((Chunk::Message(data.to_vec()), target_pkey.clone()));
    self
  }

  /// Adds a hop to the packet, so that when it reaches the node with the right skey, it'll get forwarded along the given path.
  pub fn add_hop(mut self, path: String, node_pkey: &PublicKey) -> Packet {
    self.chunks.push((Chunk::Transport(path), node_pkey.clone()));
    self
  }

  /// Serializes the packet into a sendable format.
  pub(crate) fn into_bytes(self) -> fail::Result<Vec<u8>> {
    let packet: Vec<_> = match self.signing_key {
      Some(skey) => self
        .chunks
        .into_iter()
        .map(|(c, k)| c.encrypt_and_sign(k, &skey))
        .collect(),
      None => self.chunks.into_iter().map(|(c, k)| c.encrypt(k)).collect(),
    };
    bincode::serialize(&packet).map_err(|e| fail::MesherFail::Other(Box::new(e)))
  }

  /// Given a packet and all of our secret keys, decrypt as many chunks as possible.
  ///
  /// No error is raised if no chunks could be decrypted; you just get a Vec entirely
  /// composed of [`Chunk::Encrypted`](enum.Chunk.html#variant.Encrypted).
  ///
  /// See [`Chunk::decrypt`](enum.Chunk.html#method.decrypt) for more information.
  pub(crate) fn from_bytes(packet: &[u8], keys: &[SecretKey]) -> fail::Result<Vec<Chunk>> {
    bincode::deserialize::<Vec<Vec<u8>>>(packet)
      .map(|packet| packet.into_iter().map(|c| Chunk::decrypt(c, keys)).collect())
      .map_err(|_| fail::MesherFail::InvalidPacket)
  }

  /// Same as [`Packet::from_bytes`](#method.from_bytes) but only decrypts chunks signed with one of the valid keys.
  pub(crate) fn from_signed_bytes(
    packet: &[u8],
    keys: &[SecretKey],
    sender_keys: &[PublicKey],
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
    let (sk1, pk1) = SecretKey::generate().pair();
    let (sk2, pk2) = SecretKey::generate().pair();

    let packet = Packet::unsigned()
      .add_hop("hello".to_owned(), &pk1)
      .add_message(&[1, 2, 3], &pk2)
      .into_bytes()
      .expect("Failed to serialize packet");

    let dec1 = Packet::from_bytes(&packet, &[sk1]).expect("Failed to deserialize packets");
    assert_eq!(dec1[0], Chunk::Transport("hello".to_owned()));
    assert!(matches!(dec1[1], Chunk::Encrypted(_)));

    let dec2 = Packet::from_bytes(&packet, &[sk2]).expect("Failed to deserialize packets");
    assert!(matches!(dec2[0], Chunk::Encrypted(_)));
    assert_eq!(dec2[1], Chunk::Message(vec![1, 2, 3]));
  }

  #[test]
  fn signed_serialized_deserializable() {
    let (sks, pks) = SecretKey::generate().pair();
    let (sk1, pk1) = SecretKey::generate().pair();
    let (sk2, pk2) = SecretKey::generate().pair();

    let packet = Packet::signed(sks)
      .add_hop("hello".to_owned(), &pk1)
      .add_message(&[1, 2, 3], &pk2)
      .into_bytes()
      .expect("Failed to serialize packet");

    let dec1 = Packet::from_signed_bytes(&packet, &[sk1], &[pks.clone()]).expect("Failed to deserialize packets");
    assert_eq!(dec1[0], Chunk::Transport("hello".to_owned()));
    assert!(matches!(dec1[1], Chunk::Encrypted(_)));

    let dec2 = Packet::from_signed_bytes(&packet, &[sk2], &[pks]).expect("Failed to deserialize packets");
    assert!(matches!(dec2[0], Chunk::Encrypted(_)));
    assert_eq!(dec2[1], Chunk::Message(vec![1, 2, 3]));
  }
}
