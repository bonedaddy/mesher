use std::io::{stdin, Read};

use mesher::prelude::*;
use mesher_basic::TCP;

fn get_pkey(s: &str) -> Result<PublicKey, &'static str> {
  if s.len() != 128 {
    return Err("Must be 128 bytes exactly!");
  }
  let mut bytes = [0; 64];
  for i in 0..bytes.len() {
    bytes[i] = u8::from_str_radix(&s[i*2..i*2 + 2], 16).expect("Invalid hex");
  }
  PublicKey::load(bytes).map_err(|_| "Pkey is invalid")
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

  println!("\n---\nSending {} bytes...", data.len());
  let mut m = Mesher::unsigned(vec![SecretKey::generate()]);
  m.add_transport::<TCP>("tcp").expect("Failed to add TCP transport");
  m.launch(Packet::unsigned().add_message(&data, &pkey), &format!("tcp:{}", sock))
    .expect("Failed to send data");
  println!("Sent! Did you see it get received?");
}
