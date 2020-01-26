const MAGIC: &[u8] = &[0x6d, 0x65, 0x73, 0x68]; // "mesh" in ASCII

#[derive(Debug)]
pub enum Chunk {
  Message(Vec<u8>),
  Transport(String),
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

  fn decrypt(bytes: Vec<u8>, keys: &[crate::SecretKey]) -> Result<Chunk, Vec<u8>> {
    for key in keys {
      if let Ok(dec) = Self::decrypt_onekey(&bytes, key) {
        return Ok(dec);
      }
    }
    Err(bytes)
  }
}

pub fn assemble(message: &[u8], route: crate::Route, own_pkey: crate::PublicKey) -> Vec<u8> {
  println!(
    "Assembling packet for {:?} to go along {:?}",
    message, route
  );

  let mut chunks = vec![
    (Chunk::Message(message.to_vec()), route.target.clone()),
    (Chunk::Transport(route.first_hop), own_pkey),
  ];
  for (transport, key) in route.transports {
    chunks.push((Chunk::Transport(transport), key));
  }

  println!("Chunks are:");
  for chunk in chunks.iter() {
    println!("- {:?}", chunk);
  }

  let mut packet = vec![];
  for (chunk, key) in chunks.into_iter() {
    packet.push(chunk.encrypt(key));
  }
  bincode::serialize(&packet).expect("Serialiation failed")
}

pub fn disassemble(packet: &[u8], keys: &[crate::SecretKey]) -> Vec<Result<Chunk, Vec<u8>>> {
  let packet: Vec<Vec<u8>> = match bincode::deserialize(packet) {
    Ok(v) => v,
    Err(e) => {
      println!("Errored with {:?}", e);
      return vec![];
    }
  };
  packet
    .into_iter()
    .map(|c| Chunk::decrypt(c, keys))
    .collect()
}
