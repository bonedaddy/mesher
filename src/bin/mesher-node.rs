use mesher::*;

fn make_mesher(name: &str) -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of(name)]);
  m.add_transport::<transports::debug::Printer>("print").expect("failed to add printer");
  // m.add_transport::<transports::debug::Mock>("mock").expect("failed to add mock");
  // m.listen_on(&format!("mock:{}", name))
  m
}

fn main() {
  let m_root = make_mesher("root");
  let m_n1 = make_mesher("n1");
  let m_n2 = make_mesher("n2");
  let m_target = make_mesher("target");
  let meshers = &mut [m_root, m_n1, m_n2, m_target];
  for mesher in meshers.iter_mut() {
    print!("\n\n");
    mesher
      .send(
        &[1, 2, 3],
        Route::to(PublicKey::of("target"), "print:sendfirsthop")
          .with_transport(&PublicKey::of("n1"), "print:sendpath1")
          .with_transport(&PublicKey::of("n2"), "print:sendpath2"),
      )
      .expect("Failed to send");
    for recv in mesher.recv().expect("Failed to receive") {
      println!("received: {:?}", recv.contents());
    }
  }
}
