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
fn send_with_reply_signed() {
  let (signing_pk, signing_sk) = sign::gen_keypair();
  let (mut sender, sender_pk) = make_mesher("sender", &signing_pk);
  let (mut receiver, receiver_pk) = make_mesher("receiver", &signing_pk);

  let mut packet = Packet::signed(signing_sk.clone());
  packet.add_hop("inmem:receiver".to_owned(), &sender_pk);
  let mut rh = packet.add_reply_path().expect("Failed to add reply path");
  rh.add_hop("inmem:sender".to_owned(), &receiver_pk);
  rh.use_for_message(&[1], &receiver_pk);

  sender.launch(packet).expect("Failed to send message");

  let messages = receiver.receive().expect("Failed to receive message");
  let message = &messages[0];
  assert_eq!(&[1], message.contents());

  let mut reply_packet = Packet::signed(signing_sk.clone());
  reply_packet.reply_to(&message).expect("message had no reply path");
  reply_packet.add_message(&[2], &sender_pk);

  receiver.launch(reply_packet).expect("failed to send reply");

  let replies = sender.receive().expect("Failed to receive reply");
  let reply = &replies[0];
  assert_eq!(&[2], reply.contents());
}
