#[derive(Debug, Clone)]
pub struct SimpleRoute {
  target: crate::PublicKey,
  first_hop: String,
  transports: Vec<(String, crate::PublicKey)>,
  // TODO: Replies
}

impl SimpleRoute {
  pub fn to(target_key: &crate::PublicKey, first_hop: &str) -> SimpleRoute {
    SimpleRoute {
      target: target_key.clone(),
      first_hop: first_hop.to_owned(),
      transports: Vec::new(),
    }
  }
  
  pub fn add_hop(mut self, node_key: &crate::PublicKey, path: &str) -> SimpleRoute {
    self.transports.push((path.to_owned(), node_key.clone()));
    self
  }
}

const MAGIC: &[u8] = &[0x6d, 0x65, 0x73, 0x68]; // "mesh" in ASCII

#[derive(Debug)]
pub enum Chunk {
  Message(Vec<u8>),
  Transport(String),
  // Reply(...),
  Encrypted(Vec<u8>),
}

// TODO: real crypto
impl Chunk {
  fn encrypt(self, key: crate::PublicKey) -> Vec<u8> {
    let mut b = MAGIC.to_vec();
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

  fn decrypt_onekey(bytes: &[u8], key: &crate::SecretKey) -> Result<Chunk, ()> {
    let mut attempt_dec = key.decrypt(bytes)?;
    if attempt_dec.len() < 5 || &attempt_dec[0..4] != MAGIC {
      return Err(());
    }
    match attempt_dec[4] {
      0 => Ok(Chunk::Message(attempt_dec.drain(5..).collect())),
      1 => Ok(Chunk::Transport(
        String::from_utf8(attempt_dec.drain(5..).collect()).map_err(|_| ())?,
      )),
      _ => Err(()),
    }
  }

  fn decrypt(bytes: Vec<u8>, keys: &[crate::SecretKey]) -> Chunk {
    for key in keys {
      if let Ok(dec) = Self::decrypt_onekey(&bytes, key) {
        return dec;
      }
    }
    Chunk::Encrypted(bytes)
  }
}

pub struct Packet {
  chunks: Vec<(Chunk, crate::PublicKey)>,
}

impl Packet {
  pub fn new() -> Packet {
    Packet { chunks: vec![] }
  }

  pub(crate) fn along_route(message: &[u8], route: SimpleRoute, self_pkey: &crate::PublicKey) -> Packet {
    let mut this = Packet::new().add_message(message, &route.target).add_hop(route.first_hop, self_pkey);
    for (transport, key) in route.transports {
      this = this.add_hop(transport, &key);
    }
    this
  }

  pub fn add_message(mut self, data: &[u8], target_pkey: &crate::PublicKey) -> Packet {
    self.chunks.push((Chunk::Message(data.to_vec()), target_pkey.clone()));
    self
  }

  pub fn add_hop(mut self, path: String, node_pkey: &crate::PublicKey) -> Packet {
    self.chunks.push((Chunk::Transport(path), node_pkey.clone()));
    self
  }

  pub fn into_bytes(self) -> Result<Vec<u8>, crate::TransportFail> {
    let packet = self.chunks.into_iter().map(|(c, k)| c.encrypt(k)).collect::<Vec<_>>();
    bincode::serialize(&packet).map_err(|e| crate::TransportFail::Other(Box::new(e)))
  }

  pub fn from_bytes(packet: &[u8], keys: &[crate::SecretKey]) -> Result<Vec<Chunk>, crate::TransportFail> {
    bincode::deserialize::<Vec<Vec<u8>>>(packet)
    .map(|packet| packet.into_iter().map(|c| Chunk::decrypt(c, keys)).collect())
    .map_err(|_| crate::TransportFail::InvalidPacket)
  }
}
