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

pub fn assemble(
  message: &[u8],
  route: crate::Route,
  own_pkey: crate::PublicKey,
) -> Result<Vec<u8>, crate::TransportFail> {
  let mut chunks = vec![
    (Chunk::Message(message.to_vec()), route.target.clone()),
    (Chunk::Transport(route.first_hop), own_pkey),
  ];
  for (transport, key) in route.transports {
    chunks.push((Chunk::Transport(transport), key));
  }

  let mut packet = vec![];
  for (chunk, key) in chunks.into_iter() {
    packet.push(chunk.encrypt(key));
  }
  bincode::serialize(&packet).map_err(|e| crate::TransportFail::Other(Box::new(e)))
}

pub fn disassemble(
  packet: &[u8],
  keys: &[crate::SecretKey],
) -> Result<Vec<Chunk>, crate::TransportFail> {
  bincode::deserialize::<Vec<Vec<u8>>>(packet)
    .map(|packet| {
      packet
        .into_iter()
        .map(|c| Chunk::decrypt(c, keys))
        .collect()
    })
    .map_err(|_| crate::TransportFail::InvalidPacket)
}
