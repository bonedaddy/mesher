use std::io::{stdin, Read};

use mesher::{prelude::*, packet::SimpleRoute};
use mesher_standard::TCP;

fn main() {
  let mut args = std::env::args().skip(1);
  let sock = args.next().unwrap_or("[::1]:18540".to_owned());

  println!("Enter the data to send to {}, then send EOF when done.", sock);

  let mut data = vec![];
  stdin()
    .lock()
    .read_to_end(&mut data)
    .expect("Failed to read from STDIN");

  println!("\n---\nSending {} bytes...", data.len());
  let mut m = Mesher::unsigned(vec![unsafe { SecretKey::of("who cares") }]);
  m.add_transport::<TCP>("tcp").expect("Failed to add TCP transport");
  m.send(&data, SimpleRoute::to(&unsafe { PublicKey::of("receiver") }, &format!("tcp:{}", sock)))
    .expect("Failed to send data");
  println!("Sent! Did you see it get received?");
}
