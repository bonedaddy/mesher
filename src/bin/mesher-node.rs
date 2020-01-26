use mesher::*;

fn make_mesher(name: &str) -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of(name)]);
  m.add_transport::<transports::debug::Printer>("print");
  // m.add_transport::<transports::debug::Mock>("mock");
  m
}

fn main() {
  let m_root = make_mesher("root");
  let m_n1 = make_mesher("n1");
  let m_n2 = make_mesher("n2");
  let m_target = make_mesher("target");
  let mut meshers = &mut [m_root, m_n1, m_n2, m_target];
  for mut mesher in meshers.iter_mut() {
    print!("\n\n");
    mesher
      .send(
        &[1, 2, 3],
        Route::to(PublicKey::of("target"), "debug:sendfirsthop")
          .with_transport(&PublicKey::of("n1"), "debug:sendpath1")
          .with_transport(&PublicKey::of("n2"), "debug:sendpath2"),
      )
      .expect("Failed to send");
    for recv in mesher.recv().expect("Failed to receive") {
      println!("received: {:?}", recv.contents());
    }
  }
}
