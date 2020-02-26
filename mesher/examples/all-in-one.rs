use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn make_mesher(name: &str) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<InMemory>("mock").expect("failed to add mock");
  m.listen_on(&format!("mock:{}", name)).expect("failed to listen");
  (m, pk)
}

fn main() {
  let (mut m_root, _) = make_mesher("root");
  let (mut m_n1, k_n1) = make_mesher("n1");
  let (m_n2, k_n2) = make_mesher("n2");
  let (m_target, k_target) = make_mesher("target");
  let root_pkt = {
    let mut packet = Packet::unsigned();
    packet.add_message(&[1], &k_n2);
    packet
  };
  m_root
    .launch(root_pkt, "mock:n2")
    .expect("Failed to send 1");
  let n1_pkt = {
    let mut packet = Packet::unsigned();
    packet.add_message(&[2], &k_n2);
    packet.add_hop("mock:target".to_owned(), &k_n2);
    packet
  };
  m_n1
    .launch(n1_pkt, "mock:n2")
    .expect("Failed to send 2");
  let n2_pkt = {
    let mut packet = Packet::unsigned();
    packet.add_message(&[3], &k_target);
    packet.add_hop("mock:n2".to_owned(), &k_n1);
    packet.add_hop("mock:target".to_owned(), &k_n2);
    packet
  };
  m_root
    .launch(n2_pkt, "mock:n1")
    .expect("Failed to send 3");
  println!("Sent messages! Running along pipeline...");
  for mesher in &mut [m_root, m_n1, m_n2, m_target] {
    let recvd = mesher.recv().expect("Failed to receive");
    println!("Received {} message(s)", recvd.len());
    for recv in recvd.into_iter() {
      println!("- {:?}", recv.contents());
    }
  }
}
