use mesher::prelude::*;
use mesher_basic::TCP;

fn main() {
  let mut args = std::env::args().skip(1);
  let sock = args.next().unwrap_or("[::1]:18540".to_owned());

  println!("Listening for data on {}", sock);

  let mut m = Mesher::unsigned(vec![unsafe { SecretKey::of("receiver") }]);
  m.add_transport::<TCP>("tcp").expect("Failed to add required transport");
  m.listen_on(&format!("tcp:{}", sock))
    .expect("Failed to add listener for messages");

  loop {
    let recvd = m.recv().expect("Failed to receive messages");
    for msg in recvd {
      let contents = msg.contents();
      match std::str::from_utf8(contents) {
        Ok(s) if s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) => {
          println!("Text message received ({} chars):", s.len());
          println!("{}", s);
        }
        _ => {
          println!("Binary message received ({} bytes):", contents.len());
          for (i, byte) in contents.iter().enumerate() {
            print!("{:02x}", byte);
            if i % 40 == 39 {
              println!();
            }
          }
        }
      };
      println!("\n---");
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  }
}