use mesher::prelude::*;
use mesher_standard::TCP;

use std::{thread::sleep, time::Duration};

fn make_sender() -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of("sender")]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m
}

fn make_bouncer() -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of("bouncer")]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m.listen_on("tcp:[::1]:18550").expect("Failed to listen on port");
  m
}

fn make_receiver() -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of("receiver")]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m.listen_on("tcp:[::1]:18540").expect("Failed to listen on port");
  m
}

const MESSAGES: &[&str] = &["Hello", "This is a TCP demo", "Goodbye"];

fn main() {
  let mut m1 = make_sender();
  let mut mb = make_bouncer();
  let mut m2 = make_receiver();
  let path =
    SimpleRoute::to(&PublicKey::of("receiver"), "tcp:[::1]:18550").add_hop(&PublicKey::of("bouncer"), "tcp:[::1]:18540");

  for message in MESSAGES {
    m1.send(message.as_bytes(), path.clone()).expect("Failed to send");
    println!("Message sent: {}", message);
  }

  sleep(Duration::from_millis(100));
  let _ = mb.recv();
  sleep(Duration::from_millis(100));

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
