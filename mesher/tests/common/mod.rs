use mesher::debug_transports::InMemory;
use mesher::prelude::*;

#[allow(dead_code)]
pub fn make_signed(name: &str, sender_pkey: &sign::PublicKey) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::signed(vec![sk], vec![sender_pkey.clone()]);
  m.add_transport::<InMemory>("inmem").expect("failed to add mock");
  m.listen_on(&format!("inmem:{}", name)).expect("failed to listen");
  (m, pk)
}

#[allow(dead_code)]
pub fn make_unsigned(name: &str) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<InMemory>("inmem").expect("failed to add mock");
  m.listen_on(&format!("inmem:{}", name)).expect("failed to listen");
  (m, pk)
}
