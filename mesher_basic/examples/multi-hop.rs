use mesher::prelude::*;
use mesher_basic::TCP;

use std::{thread::sleep, time::Duration};

fn make_sender() -> Mesher {
  let mut m = Mesher::unsigned(vec![]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m
}

fn make_bouncer() -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m.listen_on("tcp:localhost:18550").expect("Failed to listen on port");
  (m, pk)
}

fn make_receiver() -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  m.listen_on("tcp:localhost:18540").expect("Failed to listen on port");
  (m, pk)
}

const MESSAGES: &[&str] = &["Hello", "This is a TCP demo", "Goodbye"];

fn main() {
  let mut m1 = make_sender();
  let (mut mb, mbk) = make_bouncer();
  let (mut m2, m2k) = make_receiver();

  for message in MESSAGES {
    let mut packet = Packet::unsigned();
    packet.add_message(message.as_bytes(), &m2k);
    packet.add_hop("tcp:localhost:18540".to_owned(), &mbk);
    m1.launch(packet, "tcp:localhost:18550").expect("Faield to send");
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
