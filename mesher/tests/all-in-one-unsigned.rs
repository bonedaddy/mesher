use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn make_mesher(name: &str) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<InMemory>("inmem").expect("failed to add mock");
  m.listen_on(&format!("inmem:{}", name)).expect("failed to listen");
  (m, pk)
}

#[test]
fn direct() {
  let (mut root, root_pk) = make_mesher("direct_root");
  let (mut dest, dest_pk) = make_mesher("direct_dest");

  let mut packet = Packet::unsigned();
  packet.add_hop("inmem:direct_dest".to_owned(), &root_pk);
  packet.add_message(&[1], &dest_pk);

  root.launch(packet).expect("Failed to send");

  let msgs = dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}

#[test]
fn one_hop() {
  let (mut root, root_pk) = make_mesher("onehop_root");
  let (mut n1, n1_pk) = make_mesher("onehop_n1");
  let (mut dest, dest_pk) = make_mesher("onehop_dest");

  let mut packet = Packet::unsigned();
  packet.add_hop("inmem:onehop_n1".to_owned(), &root_pk);
  packet.add_hop("inmem:onehop_dest".to_owned(), &n1_pk);
  packet.add_message(&[1], &dest_pk);

  root.launch(packet).expect("Failed to send");

  // will bounce the message along to dest
  n1.receive().expect("Failed to receive");

  let msgs = dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}

#[test]
fn two_hops() {
  let (mut root, root_pk) = make_mesher("twohops_root");
  let (mut n1, n1_pk) = make_mesher("twohops_n1");
  let (mut n2, n2_pk) = make_mesher("twohops_n2");
  let (mut dest, dest_pk) = make_mesher("twohops_dest");

  let mut packet = Packet::unsigned();
  packet.add_hop("inmem:twohops_n1".to_owned(), &root_pk);
  packet.add_hop("inmem:twohops_n2".to_owned(), &n1_pk);
  packet.add_hop("inmem:twohops_dest".to_owned(), &n2_pk);
  packet.add_message(&[1], &dest_pk);

  root.launch(packet).expect("Failed to send");

  // will bounce the message along to n2
  n1.receive().expect("Failed to receive");
  // will bounce the message along to dest
  n2.receive().expect("Failed to receive");

  let msgs = dest.receive().expect("Failed to receive").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();

  assert_eq!(msgs, vec![vec![1]]);
}
