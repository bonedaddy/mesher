use mesher::debug_transports::InMemory;
use mesher::prelude::*;

fn main() {
  let (r1_pk, r1_sk) = encrypt::gen_keypair();
  let (r2_pk, r2_sk) = encrypt::gen_keypair();

  let mut sender = Mesher::unsigned(vec![]);
  sender
    .add_transport::<InMemory>("tcp")
    .expect("Failed to add transport");
  let mut recvr1 = Mesher::unsigned(vec![r1_sk]);
  recvr1
    .add_transport::<InMemory>("tcp")
    .expect("Failed to add transport");
  recvr1.listen_on("tcp:localhost:18540").expect("Failed to listen");
  let mut recvr2 = Mesher::unsigned(vec![r2_sk]);
  recvr2
    .add_transport::<InMemory>("tcp")
    .expect("Failed to add transport");
  recvr2.listen_on("tcp:localhost:18550").expect("Failed to listen");

  let packet = Packet::unsigned()
    .add_message(&[1], &r1_pk)
    .add_message(&[2], &r2_pk)
    .add_hop("tcp:localhost:18550".to_owned(), &r1_pk);

  sender.launch(packet, "tcp:localhost:18540").expect("failed to launch");

  for mut r in vec![recvr1, recvr2].into_iter() {
    loop {
      let recvd = r.recv().expect("failed to recv");
      if recvd.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(50));
        continue;
      }
      for msg in recvd {
        println!("got {:?}", msg);
      }
      break;
    }
  }
}
