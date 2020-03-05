use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn make_mesher(name: &str, pkey: &sign::PublicKey) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut mesh = Mesher::signed(vec![sk], vec![pkey.clone()]);
  mesh.add_transport::<InMemory>("inmem").expect("Failed to add transport");
  mesh.listen_on(&format!("inmem:{}", name)).expect("Failed to listen");
  (mesh, pk)
}

#[test]
fn direct() {
  let (sender_pkey, sender_skey) = sign::gen_keypair();

  let (mut sender, _) = make_mesher("direct_sender", &sender_pkey);
  let (mut dest1, dest1_pk) = make_mesher("direct_dest1", &sender_pkey);
  let (mut dest2, dest2_pk) = make_mesher("direct_dest2", &sender_pkey);

  let mut packet = Packet::signed(sender_skey);
  packet.add_message(&[1], &dest1_pk);
  packet.add_message(&[2], &dest2_pk);

  sender.launch(packet.clone(), "inmem:direct_dest1").expect("failed to launch to dest1");
  sender.launch(packet.clone(), "inmem:direct_dest2").expect("failed to launch to dest2");

  let recvd1 = dest1.recv().expect("failed to recv at 1");
  assert_eq!(vec![vec![1]], recvd1.iter().map(|m| m.contents()).collect::<Vec<_>>());

  let recvd2 = dest2.recv().expect("failed to recv at 2");
  assert_eq!(vec![vec![2]], recvd2.iter().map(|m| m.contents()).collect::<Vec<_>>());
}

#[test]
fn one_hop() {
  let (sender_pkey, sender_skey) = sign::gen_keypair();

  let (mut sender, _) = make_mesher("onehop_sender", &sender_pkey);
  let (mut im, im_pk) = make_mesher("onehop_im", &sender_pkey);
  let (mut dest1, dest1_pk) = make_mesher("onehop_dest1", &sender_pkey);
  let (mut dest2, dest2_pk) = make_mesher("onehop_dest2", &sender_pkey);

  let mut packet = Packet::signed(sender_skey);
  packet.add_hop("inmem:onehop_dest1".to_owned(), &im_pk);
  packet.add_hop("inmem:onehop_dest2".to_owned(), &im_pk);
  packet.add_message(&[1], &dest1_pk);
  packet.add_message(&[2], &dest2_pk);

  sender.launch(packet.clone(), "inmem:onehop_im").expect("failed to launch to dest2");

  // will bounce the message along to dest1 and dest2
  im.recv().expect("failed to recv at im");

  let recvd1 = dest1.recv().expect("failed to recv at 1");
  assert_eq!(vec![vec![1]], recvd1.iter().map(|m| m.contents()).collect::<Vec<_>>());

  let recvd2 = dest2.recv().expect("failed to recv at 2");
  assert_eq!(vec![vec![2]], recvd2.iter().map(|m| m.contents()).collect::<Vec<_>>());
}

#[test]
fn two_hop() {
  let (sender_pkey, sender_skey) = sign::gen_keypair();

  let (mut sender, _) = make_mesher("twohops_sender", &sender_pkey);
  let (mut im1, im1_pk) = make_mesher("twohops_im1", &sender_pkey);
  let (mut im2, im2_pk) = make_mesher("twohops_im2", &sender_pkey);
  let (mut dest1, dest1_pk) = make_mesher("twohops_dest1", &sender_pkey);
  let (mut dest2, dest2_pk) = make_mesher("twohops_dest2", &sender_pkey);

  let mut packet = Packet::signed(sender_skey);
  packet.add_hop("inmem:twohops_im2".to_owned(), &im1_pk);
  packet.add_hop("inmem:twohops_dest1".to_owned(), &im2_pk);
  packet.add_hop("inmem:twohops_dest2".to_owned(), &im2_pk);
  packet.add_message(&[1], &dest1_pk);
  packet.add_message(&[2], &dest2_pk);

  sender.launch(packet.clone(), "inmem:twohops_im1").expect("failed to launch to dest2");

  // will bounce the message along to im2
  im1.recv().expect("failed to recv at im1");
  // will bounce the message along to dest1 and dest2
  im2.recv().expect("failed to recv at im2");

  let recvd1 = dest1.recv().expect("failed to recv at 1");
  assert_eq!(vec![vec![1]], recvd1.iter().map(|m| m.contents()).collect::<Vec<_>>());

  let recvd2 = dest2.recv().expect("failed to recv at 2");
  assert_eq!(vec![vec![2]], recvd2.iter().map(|m| m.contents()).collect::<Vec<_>>());
}
