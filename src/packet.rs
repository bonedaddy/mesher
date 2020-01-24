const MAGIC: &'static [u8] = &[0x6d, 0x65, 0x73, 0x68]; // mesh ASCII

fn insert_usize(into: &mut Vec<u8>, val: usize) {
  into.push(val as u8);
  into.push((val >> 8) as u8);
  into.push((val >> 16) as u8);
  into.push((val >> 24) as u8);
}

pub enum PacketChunk {
  Message(Vec<u8>),
  Transport(String),
  Reply(crate::PublicKey),
}

impl PacketChunk {
  fn encrypt(self, key: crate::PublicKey) -> Vec<u8> {
    let raw = match self {
      PacketChunk::Message(mut m) => {
        let mut b = MAGIC.to_vec();
        b.push(0u8);
        insert_usize(&mut b, m.len());
        b.append(&mut m);
        b
      },
      PacketChunk::Reply(k) => {
        let mut b = MAGIC.to_vec();
        b.push(1u8);
        insert_usize(&mut b, k.0);
        b
      },
      PacketChunk::Transport(t) => {
        let mut b = MAGIC.to_vec();
        b.push(2u8);
        insert_usize(&mut b, t.len());
        b.append(&mut t.into_bytes());
        b
      }
    };
    // TODO: encrypt
    raw
  }
}

pub fn assemble_packet(message: &[u8], route: crate::Route, own_pkey: crate::PublicKey) -> Vec<u8> {
  println!("Assembling packet for {:?} to go along {:?}", message, route);
  let mut chunks = vec![
    (PacketChunk::Message(message.to_vec()), route.target),
    (PacketChunk::Transport(route.first_hop), own_pkey),
  ];
  for (transport, key) in route.transports {
    chunks.push((PacketChunk::Transport(transport), key));
  }
  // TODO: Replies
  vec![]
}
