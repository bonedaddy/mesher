use mesher::prelude::*;

use mesher_debug::InMemory;

fn main() {
  let mut sender = Mesher::unsigned(vec![unsafe { SecretKey::of("s") }]);
  sender
    .add_transport::<InMemory>("tcp")
    .expect("Failed to add transport");
  let mut recvr1 = Mesher::unsigned(vec![unsafe { SecretKey::of("r1") }]);
  recvr1
    .add_transport::<InMemory>("tcp")
    .expect("Failed to add transport");
  recvr1.listen_on("tcp:localhost:18540").expect("Failed to listen");
  let mut recvr2 = Mesher::unsigned(vec![unsafe { SecretKey::of("r2") }]);
  recvr2
    .add_transport::<InMemory>("tcp")
    .expect("Failed to add transport");
  recvr2.listen_on("tcp:localhost:18550").expect("Failed to listen");

  let packet = Packet::unsigned()
    .add_message(&[1], &unsafe { PublicKey::of("r1") })
    .add_message(&[2], &unsafe { PublicKey::of("r2") })
    .add_hop("tcp:localhost:18550".to_owned(), &unsafe { PublicKey::of("r1") });

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
