use mesher::*;

fn main() {
  let mut mesher = Mesher::unsigned(vec![SecretKey(3)]);
  mesher
    .add_transport::<transports::Debug>("debug")
    .expect("Failed to set scheme");
  mesher
    .send(
      &[1, 2, 3],
      Route::to(PublicKey(0), "debug:sendfirsthop")
        .with_transport(&PublicKey(1), "debug:sendpath1")
        .with_transport(&PublicKey(2), "debug:sendpath2")
    )
    .expect("Failed to send");
  for recv in mesher.recv().expect("Failed to receive") {
    println!("received: {:?}", recv.contents());
  }
}
