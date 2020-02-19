use mesher::prelude::*;
use mesher::debug_transports::InMemory;

fn make_mesher(name: &str) -> Mesher {
  let mut m = Mesher::unsigned(vec![unsafe { SecretKey::of(name) }]);
  m.add_transport::<InMemory>("mock")
    .expect("failed to add mock");
  m.listen_on(&format!("mock:{}", name)).expect("failed to listen");
  m
}

fn main() {
  let mut m_root = make_mesher("root");
  let mut m_n1 = make_mesher("n1");
  let m_n2 = make_mesher("n2");
  let m_target = make_mesher("target");
  m_root
    .launch(
      Packet::unsigned().add_message(&[1], &unsafe { PublicKey::of("n2") }),
      "mock:n2",
    )
    .expect("Failed to send 1");
  m_n1
    .launch(
      Packet::unsigned()
        .add_message(&[2], &unsafe { PublicKey::of("target") })
        .add_hop("mock:target".to_owned(), &unsafe { PublicKey::of("n2") }),
      "mock:n2",
    )
    .expect("Failed to send 2");
  m_root
    .launch(
      Packet::unsigned()
        .add_message(&[3], &unsafe { PublicKey::of("target") })
        .add_hop("mock:n2".to_owned(), &unsafe { PublicKey::of("n1") })
        .add_hop("mock:target".to_owned(), &unsafe { PublicKey::of("n2") }),
      "mock:n1",
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
