use mesher::debug_transports::InMemory;
use mesher::prelude::*;

#[test]
fn all_messages_received() {
  let mut sender = Mesher::unsigned(vec![]);
  sender
    .add_transport::<InMemory>("inmem")
    .expect("Failed to add transport to sender");

  let (r1_pk, r1_sk) = encrypt::gen_keypair();
  let mut recvr1 = Mesher::unsigned(vec![r1_sk]);
  recvr1
    .add_transport::<InMemory>("inmem")
    .expect("Failed to add transport to recvr1");
  recvr1.listen_on("inmem:r1").expect("Failed to listen");

  let (r2_pk, r2_sk) = encrypt::gen_keypair();
  let mut recvr2 = Mesher::unsigned(vec![r2_sk]);
  recvr2
    .add_transport::<InMemory>("inmem")
    .expect("Failed to add transport to recvr2");
  recvr2.listen_on("inmem:r2").expect("Failed to listen");

  let mut packet = Packet::unsigned();
  packet.add_message(&[1], &r1_pk);
  packet.add_message(&[2], &r2_pk);

  sender.launch(packet.clone(), "inmem:r1").expect("failed to launch to r1");
  sender.launch(packet.clone(), "inmem:r2").expect("failed to launch to r2");

  let recvd1 = recvr1.recv().expect("failed to recv at 1");
  assert_eq!(vec![vec![1]], recvd1.iter().map(|m| m.contents()).collect::<Vec<_>>());

  let recvd2 = recvr2.recv().expect("failed to recv at 2");
  assert_eq!(vec![vec![2]], recvd2.iter().map(|m| m.contents()).collect::<Vec<_>>());
}
