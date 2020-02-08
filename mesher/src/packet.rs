use crate::prelude::*;

/// One piece of a packet.
#[derive(Debug)]
pub(crate) enum Chunk {
  /// A message to pass back to the [`Mesher`][1]
  /// 
  ///  [1]: ../struct.Mesher.html
  Message(Vec<u8>),
  /// A path to send this packet along
  Transport(String),
  /// A chunk we couldn't decrypt (meant for someone else)
  Encrypted(Vec<u8>),
}

impl Chunk {
  /// Convert this chunk to a raw byte form, then encrypt those to the public key.
  /// 
  /// Should be considered a black box, as the format may change in the future.
  /// It will, of course, always be decryptable (assuming the keys match) by [`Chunk::decrypt`][1]
  /// 
  ///  [1]: #method.decrypt
  fn encrypt(self, key: PublicKey) -> Vec<u8> {
    let mut b = vec![];
    let raw = match self {
      Chunk::Message(mut m) => {
        b.push(0u8);
        b.append(&mut m);
        b
      }
      Chunk::Transport(t) => {
        b.push(1u8);
        b.append(&mut t.into_bytes());
        b
      }
      Chunk::Encrypted(v) => return v,
    };
    key.encrypt(&raw)
  }

  /// Attempt a decryption, but with one key.
  /// Separating this into its own function simplifies the real decryption code.
  fn decrypt_onekey(bytes: &[u8], key: &SecretKey) -> Result<Chunk, ()> {
    let mut attempt_dec = key.decrypt(bytes)?;
    if attempt_dec.is_empty() {
      return Err(());
    }
    match attempt_dec[0] {
      0 => Ok(Chunk::Message(attempt_dec.drain(1..).collect())),
      1 => Ok(Chunk::Transport(
        String::from_utf8(attempt_dec.drain(1..).collect()).map_err(|_| ())?,
      )),
      _ => Err(()),
    }
  }

  /// Decrypt a chunk of bytes with all of our keys.
  /// Returns the first chunk that was successfully decrypted.
  /// If none of them work, returns [`Chunk::Encrypted`][1].
  /// 
  /// Expect the input format to this to be a black box.
  /// Give it things encrypted with [`Chunk::encrypt`][2].
  /// 
  ///  [1]: #variant.Encrypted
  ///  [2]: #method.encrypt
  fn decrypt(bytes: Vec<u8>, keys: &[SecretKey]) -> Chunk {
    for key in keys {
      if let Ok(dec) = Self::decrypt_onekey(&bytes, key) {
        return dec;
      }
    }
    Chunk::Encrypted(bytes)
  }
}

/// Represents a packet to be sent out.
/// 
/// Note that each piece of the packet is associated with a key.
/// The keys don't have to be unique -- more than one piece can be associated with a single key.
/// For example, if a node is meant to both receive a message and transport the packet further, those two might be encrypted with the same key.
#[derive(Default)]
pub struct Packet {
  chunks: Vec<(Chunk, PublicKey)>,
}

impl Packet {
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
    let packet = self.chunks.into_iter().map(|(c, k)| c.encrypt(k)).collect::<Vec<_>>();
    bincode::serialize(&packet).map_err(|e| fail::MesherFail::Other(Box::new(e)))
  }

  /// Given a packet and all of our secret keys, decrypt as many chunks as possible.
  /// 
  /// No error is raised if no chunks could be decrypted; you just get a Vec entirely
  /// composed of [`Chunk::Encrypted`][1].
  /// 
  /// See [`Chunk::decrypt`][2] for more information.
  /// 
  ///  [1]: enum.Chunk.html#variant.Encrypted
  ///  [2]: enum.Chunk.html#method.decrypt
  pub(crate) fn from_bytes(packet: &[u8], keys: &[SecretKey]) -> fail::Result<Vec<Chunk>> {
    bincode::deserialize::<Vec<Vec<u8>>>(packet)
      .map(|packet| packet.into_iter().map(|c| Chunk::decrypt(c, keys)).collect())
      .map_err(|_| fail::MesherFail::InvalidPacket)
  }
}
