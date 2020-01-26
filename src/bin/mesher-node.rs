use mesher::*;

fn main() {
  let mut mesher = Mesher::unsigned(vec![SecretKey::of("root")]);
  mesher
    .add_transport::<transports::debug::Printer>("debug")
    .expect("Failed to set scheme");
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
