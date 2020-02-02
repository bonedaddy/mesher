use mesher::prelude::*;
use mesher_standard::TCP;

use std::{thread::sleep, time::Duration};

fn make_sender() -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of("sender")]);
  m.add_transport::<TCP>("tcp")
    .expect("Failed to add transport");
  m
}

fn make_receiver() -> (Mesher, Route) {
  let mut m = Mesher::unsigned(vec![SecretKey::of("receiver")]);
  m.add_transport::<TCP>("tcp")
    .expect("Failed to add transport");
  m.listen_on("tcp:[::1]:18540")
    .expect("Failed to listen on port");
  (m, Route::to(&PublicKey::of("receiver"), "tcp:[::1]:18540"))
}

const MESSAGES: &[&str] = &["Hello", "Goodbye"];

fn main() {
  let mut m1 = make_sender();
  let (mut m2, path) = make_receiver();

  for message in MESSAGES {
    m1.send(message.as_bytes(), path.clone())
      .expect("Failed to send");
    println!("Message sent: {}", message);
    loop {
      let recvd = m2.recv().expect("Failed to recv");
      match recvd.first() {
        Some(s) => {
          let contents =
            std::str::from_utf8(s.contents()).expect("Invalid UTF-8");
          println!("Message received: {}", contents);
          break;
        }
        None => {
          println!("No message gotten yet...");
          sleep(Duration::from_millis(10));
        }
      }
    }
  }
}
