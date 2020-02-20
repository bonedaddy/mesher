use mesher::prelude::*;
use mesher_basic::TCP;

use std::{thread::sleep, time::Duration};

fn make_sender() -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::generate()]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m
}

fn make_receiver() -> (Mesher, PublicKey) {
  let (sk, pk) = SecretKey::generate().pair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m.listen_on("tcp:localhost:18540").expect("Failed to listen on port");
  (m, pk)
}

const MESSAGES: &[&str] = &["Hello", "This is a TCP demo", "Goodbye"];

fn main() {
  let mut m1 = make_sender();
  let (mut m2, m2k) = make_receiver();

  for message in MESSAGES {
    let packet = Packet::unsigned().add_message(message.as_bytes(), &m2k);
    m1.launch(packet, "tcp:localhost:18540").expect("Failed to send");
    println!("Message sent: {}", message);
  }

  let mut to_read = MESSAGES.len();
  loop {
    let recvd = m2.recv().expect("Failed to recv");
    if recvd.is_empty() {
      println!("No message gotten yet...");
      sleep(Duration::from_millis(10));
    } else {
      to_read -= recvd.len();
      for msg in recvd {
        let contents = std::str::from_utf8(msg.contents()).expect("Invalid UTF-8");
        println!("Message received: {}", contents);
      }
      if to_read == 0 {
        break;
      } else {
        println!("Only received some. Waiting for rest...");
        sleep(Duration::from_millis(10));
      }
    }
  }
  println!("Received everything!");
}
