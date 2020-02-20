use mesher::prelude::*;
use mesher::debug_transports::InMemory;

fn main() {
  let (send_sk, _) = SecretKey::generate().pair();
  let (r1_sk, r1_pk) = SecretKey::generate().pair();
  let (r2_sk, r2_pk) = SecretKey::generate().pair();

  let mut sender = Mesher::unsigned(vec![send_sk]);
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
