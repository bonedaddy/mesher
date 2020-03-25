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
fn send_with_reply() {
  let (mut m_sender, pk_sender) = make_mesher("sender");
  let (mut m_receiver, pk_receiver) = make_mesher("receiver");

  let mut packet = Packet::unsigned();
  let mut rh = packet.add_reply_path().expect("Failed to add reply path");
  rh.add_hop("inmem:sender".to_owned(), &pk_receiver);
  rh.use_for_message(&[1], &pk_receiver);

  m_sender.launch(packet, "inmem:receiver").expect("Failed to send message");

  let messages = m_receiver.receive().expect("Failed to receive message");
  let message = &messages[0];
  assert_eq!(&[1], message.contents());

  m_receiver.reply(&message, &[2], &pk_sender).expect("Failed to send reply");

  let replies = m_sender.receive().expect("Failed to receive reply");
  let reply = &replies[0];
  assert_eq!(&[2], reply.contents());
}
