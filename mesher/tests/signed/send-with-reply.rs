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
fn send_with_reply_signed() {
  let (signing_pk, signing_sk) = sign::gen_keypair();
  let (mut m_sender, sender_pk) = make_mesher("sender", &signing_pk);
  let (mut m_receiver, receiver_pk) = make_mesher("receiver", &signing_pk);

  let mut packet = Packet::signed(signing_sk.clone());
  let mut rh = packet.add_reply_path().expect("Failed to add reply path");
  rh.add_hop("inmem:sender".to_owned(), &receiver_pk);
  rh.use_for_message(&[1], &receiver_pk);

  m_sender.launch(packet, "inmem:receiver").expect("Failed to send message");

  let messages = m_receiver.receive().expect("Failed to receive message");
  let message = &messages[0];
  assert_eq!(&[1], message.contents());

  m_receiver.signed_reply(&message, &[2], &sender_pk, signing_sk.clone()).expect("Failed to send reply");

  let replies = m_sender.receive().expect("Failed to receive reply");
  let reply = &replies[0];
  assert_eq!(&[2], reply.contents());
}
