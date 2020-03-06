use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn make_mesher(name: &str) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<InMemory>("mock").expect("failed to add mock");
  m.listen_on(&format!("mock:{}", name)).expect("failed to listen");
  (m, pk)
}

#[test]
fn direct() {
  let (mut m_root, _) = make_mesher("direct_root");
  let (mut m_dest, k_dest) = make_mesher("direct_dest");

  let mut packet = Packet::unsigned();
  packet.add_message(&[1], &k_dest);

  m_root.launch(packet, "mock:direct_dest").expect("Failed to send");

  let msgs = m_dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}

#[test]
fn one_hop() {
  let (mut m_root, _) = make_mesher("onehop_root");
  let (mut m_n1, k_n1) = make_mesher("onehop_n1");
  let (mut m_dest, k_dest) = make_mesher("onehop_dest");

  let mut packet = Packet::unsigned();
  packet.add_hop("mock:onehop_dest".to_owned(), &k_n1);
  packet.add_message(&[1], &k_dest);

  m_root.launch(packet, "mock:onehop_n1").expect("Failed to send");

  // will bounce the message along to m_dest
  m_n1.receive().expect("Failed to receive");

  let msgs = m_dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}

#[test]
fn two_hops() {
  let (mut m_root, _) = make_mesher("twohops_root");
  let (mut m_n1, k_n1) = make_mesher("twohops_n1");
  let (mut m_n2, k_n2) = make_mesher("twohops_n2");
  let (mut m_dest, k_dest) = make_mesher("twohops_dest");

  let mut packet = Packet::unsigned();
  packet.add_hop("mock:twohops_n2".to_owned(), &k_n1);
  packet.add_hop("mock:twohops_dest".to_owned(), &k_n2);
  packet.add_message(&[1], &k_dest);

  m_root.launch(packet, "mock:twohops_n1").expect("Failed to send");

  // will bounce the message along to m_n2
  m_n1.receive().expect("Failed to receive");
  // will bounce the message along to m_dest
  m_n2.receive().expect("Failed to receive");

  let msgs = m_dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}
