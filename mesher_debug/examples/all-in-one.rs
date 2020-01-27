use mesher::prelude::*;

fn make_mesher(name: &str) -> Mesher {
  let mut m = Mesher::unsigned(vec![SecretKey::of(name)]);
  m.add_transport::<mesher_debug::InMemory>("mock")
    .expect("failed to add mock");
  m.listen_on(&format!("mock:{}", name))
    .expect("failed to listen");
  m
}

fn main() {
  let mut m_root = make_mesher("root");
  let mut m_n1 = make_mesher("n1");
  let m_n2 = make_mesher("n2");
  let m_target = make_mesher("target");
  m_root
    .send(&[1], Route::to(&PublicKey::of("n2"), "mock:n2"))
    .expect("Failed to send 1");
  m_n1
    .send(
      &[2],
      Route::to(&PublicKey::of("target"), "mock:n2")
        .add_hop(&PublicKey::of("n2"), "mock:target"),
    )
    .expect("Failed to send 2");
  m_root
    .send(
      &[3],
      Route::to(&PublicKey::of("target"), "mock:n1")
        .add_hop(&PublicKey::of("n1"), "mock:n2")
        .add_hop(&PublicKey::of("n2"), "mock:target"),
    )
    .expect("Failed to send 3");
  println!("Sent messages! Running along pipeline...");
  for mesher in &mut [m_root, m_n1, m_n2, m_target] {
    let recvd = mesher.recv().expect("Failed to receive");
    println!("Received {} message(s)", recvd.len());
    for recv in recvd.into_iter() {
      println!("- {:?}", recv.contents());
    }
  }
}
