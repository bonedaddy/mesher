use mesher::*;

fn make_mesher(name: &str) -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of(name)]);
  m.add_transport::<transports::debug::Mock>("mock").expect("failed to add mock");
  m.listen_on(&format!("mock:{}", name)).expect("failed to listen");
  m
}

fn main() {
  let mut m_root = make_mesher("root");
  let m_n1 = make_mesher("n1");
  let m_n2 = make_mesher("n2");
  let m_target = make_mesher("target");
  m_root
    .send(
      &[1, 2, 3],
      Route::to(PublicKey::of("target"), "mock:n1")
        .with_transport(&PublicKey::of("n1"), "mock:n2")
        .with_transport(&PublicKey::of("n2"), "mock:target"),
    )
    .expect("Failed to send");
  for mesher in &mut [m_root, m_n1, m_n2, m_target] {
    println!("---");
    for recv in mesher.recv().expect("Failed to receive") {
      println!("received: {:?}", recv.contents());
    }
  }
}
