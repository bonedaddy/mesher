use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn make_mesher(name: &str, sender_pkey: &sign::PublicKey) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::signed(vec![sk], vec![sender_pkey.clone()]);
  m.add_transport::<InMemory>("inmem").expect("failed to add mock");
  m.listen_on(&format!("inmem:{}", name)).expect("failed to listen");
  (m, pk)
}

#[test]
fn direct() {
  let (signer_pkey, signer_skey) = sign::gen_keypair();

  let (mut m_root, _) = make_mesher("direct_root", &signer_pkey);
  let (mut m_dest, k_dest) = make_mesher("direct_dest", &signer_pkey);

  let mut packet = Packet::signed(signer_skey);
  packet.add_message(&[1], &k_dest);

  m_root.launch(packet, "inmem:direct_dest").expect("Failed to send");

  let msgs = m_dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}

#[test]
fn one_hop() {
  let (signer_pkey, signer_skey) = sign::gen_keypair();

  let (mut m_root, _) = make_mesher("onehop_root", &signer_pkey);
  let (mut m_n1, k_n1) = make_mesher("onehop_n1", &signer_pkey);
  let (mut m_dest, k_dest) = make_mesher("onehop_dest", &signer_pkey);

  let mut packet = Packet::signed(signer_skey);
  packet.add_hop("inmem:onehop_dest".to_owned(), &k_n1);
  packet.add_message(&[1], &k_dest);

  m_root.launch(packet, "inmem:onehop_n1").expect("Failed to send");

  // will bounce the message along to m_dest
  m_n1.receive().expect("Failed to receive");

  let msgs = m_dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}

#[test]
fn two_hops() {
  let (signer_pkey, signer_skey) = sign::gen_keypair();

  let (mut m_root, _) = make_mesher("twohops_root", &signer_pkey);
  let (mut m_n1, k_n1) = make_mesher("twohops_n1", &signer_pkey);
  let (mut m_n2, k_n2) = make_mesher("twohops_n2", &signer_pkey);
  let (mut m_dest, k_dest) = make_mesher("twohops_dest", &signer_pkey);

  let mut packet = Packet::signed(signer_skey);
  packet.add_hop("inmem:twohops_n2".to_owned(), &k_n1);
  packet.add_hop("inmem:twohops_dest".to_owned(), &k_n2);
  packet.add_message(&[1], &k_dest);

  m_root.launch(packet, "inmem:twohops_n1").expect("Failed to send");

  // will bounce the message along to m_n2
  m_n1.receive().expect("Failed to receive");
  // will bounce the message along to m_dest
  m_n2.receive().expect("Failed to receive");

  let msgs = m_dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}
