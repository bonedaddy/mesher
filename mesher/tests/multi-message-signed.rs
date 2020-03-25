use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn make_mesher(name: &str, pkey: &sign::PublicKey) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut mesh = Mesher::signed(vec![sk], vec![pkey.clone()]);
  mesh
    .add_transport::<InMemory>("inmem")
    .expect("Failed to add transport");
  mesh.listen_on(&format!("inmem:{}", name)).expect("Failed to listen");
  (mesh, pk)
}

#[test]
fn direct() {
  let (signer_pk, signer_sk) = sign::gen_keypair();

  let (mut sender, sender_pk) = make_mesher("direct_sender", &signer_pk);
  let (mut dest1, dest1_pk) = make_mesher("direct_dest1", &signer_pk);
  let (mut dest2, dest2_pk) = make_mesher("direct_dest2", &signer_pk);

  let mut packet = Packet::signed(signer_sk);
  packet.add_hop("inmem:direct_dest1".to_owned(), &sender_pk);
  packet.add_hop("inmem:direct_dest2".to_owned(), &sender_pk);
  packet.add_message(&[1], &dest1_pk);
  packet.add_message(&[2], &dest2_pk);

  sender.launch(packet).expect("failed to launch");

  let received1 = dest1.receive().expect("failed to receive at 1");
  assert_eq!(
    vec![vec![1]],
    received1.iter().map(|m| m.contents()).collect::<Vec<_>>()
  );

  let received2 = dest2.receive().expect("failed to receive at 2");
  assert_eq!(
    vec![vec![2]],
    received2.iter().map(|m| m.contents()).collect::<Vec<_>>()
  );
}

#[test]
fn one_hop() {
  let (signer_pk, signer_sk) = sign::gen_keypair();

  let (mut sender, sender_pk) = make_mesher("onehop_sender", &signer_pk);
  let (mut im, im_pk) = make_mesher("onehop_im", &signer_pk);
  let (mut dest1, dest1_pk) = make_mesher("onehop_dest1", &signer_pk);
  let (mut dest2, dest2_pk) = make_mesher("onehop_dest2", &signer_pk);

  let mut packet = Packet::signed(signer_sk);
  packet.add_hop("inmem:onehop_im".to_owned(), &sender_pk);
  packet.add_hop("inmem:onehop_dest1".to_owned(), &im_pk);
  packet.add_hop("inmem:onehop_dest2".to_owned(), &im_pk);
  packet.add_message(&[1], &dest1_pk);
  packet.add_message(&[2], &dest2_pk);

  sender.launch(packet).expect("failed to launch");

  // will bounce the message along to dest1 and dest2
  im.receive().expect("failed to receive at im");

  let received1 = dest1.receive().expect("failed to receive at 1");
  assert_eq!(
    vec![vec![1]],
    received1.iter().map(|m| m.contents()).collect::<Vec<_>>()
  );

  let received2 = dest2.receive().expect("failed to receive at 2");
  assert_eq!(
    vec![vec![2]],
    received2.iter().map(|m| m.contents()).collect::<Vec<_>>()
  );
}

#[test]
fn two_hop() {
  let (signer_pk, signer_sk) = sign::gen_keypair();

  let (mut sender, sender_pk) = make_mesher("twohops_sender", &signer_pk);
  let (mut im1, im1_pk) = make_mesher("twohops_im1", &signer_pk);
  let (mut im2, im2_pk) = make_mesher("twohops_im2", &signer_pk);
  let (mut dest1, dest1_pk) = make_mesher("twohops_dest1", &signer_pk);
  let (mut dest2, dest2_pk) = make_mesher("twohops_dest2", &signer_pk);

  let mut packet = Packet::signed(signer_sk);
  packet.add_hop("inmem:twohops_im1".to_owned(), &sender_pk);
  packet.add_hop("inmem:twohops_im2".to_owned(), &im1_pk);
  packet.add_hop("inmem:twohops_dest1".to_owned(), &im2_pk);
  packet.add_hop("inmem:twohops_dest2".to_owned(), &im2_pk);
  packet.add_message(&[1], &dest1_pk);
  packet.add_message(&[2], &dest2_pk);

  sender.launch(packet).expect("failed to launch");

  // will bounce the message along to im2
  im1.receive().expect("failed to receive at im1");
  // will bounce the message along to dest1 and dest2
  im2.receive().expect("failed to receive at im2");

  let received1 = dest1.receive().expect("failed to receive at 1");
  assert_eq!(
    vec![vec![1]],
    received1.iter().map(|m| m.contents()).collect::<Vec<_>>()
  );

  let received2 = dest2.receive().expect("failed to receive at 2");
  assert_eq!(
    vec![vec![2]],
    received2.iter().map(|m| m.contents()).collect::<Vec<_>>()
  );
}
