use std::io::{stdin, Read};

use mesher::prelude::*;
use mesher_basic::TCP;

fn get_pkey(s: &str) -> Result<encrypt::PublicKey, &'static str> {
  if s.len() != 64 {
    return Err("Must be 64 characters/32 hex bytes exactly");
  }
  let mut bytes = [0; 32];
  for i in 0..bytes.len() {
    bytes[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect("Invalid hex");
  }
  encrypt::PublicKey::from_slice(&bytes).ok_or("Pkey is invalid")
}

fn main() {
  let mut args = std::env::args().skip(1);
  let pkey = get_pkey(&args.next().expect("Must provide key")).expect("Invalid key");
  let sock = args.next().unwrap_or("[::1]:18540".to_owned());

  println!("Enter the data to send to {}, then send EOF when done.", sock);

  let mut data = vec![];
  stdin()
    .lock()
    .read_to_end(&mut data)
    .expect("Failed to read from STDIN");

  if !data.ends_with(&['\n' as u8]) {
    println!();
  }
  println!("Sending {} bytes...", data.len());
  let (self_pk, self_sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![self_sk]);
  m.add_transport::<TCP>("tcp").expect("Failed to add TCP transport");
  let mut packet = Packet::unsigned();
  packet.add_hop(format!("tcp:{}", sock), &self_pk);
  packet.add_message(&data, &pkey);
  m.launch(packet).expect("Failed to send data");
  println!("Sent! Did you see it get received?");
}
