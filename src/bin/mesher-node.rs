use mesher::*;

fn main() {
  let mut mesher = Mesher::unsigned();
  mesher
    .add_transport::<transports::Debug>("debug")
    .expect("Failed to set scheme");
  mesher.send(
    &[1, 2, 3],
    Route::to(PublicKey(0))
      .with_transport(&PublicKey(1), "debug:sendpath1")
      .with_transport(&PublicKey(2), "debug:sendpath2")
      .reply_to(PublicKey(3))
      .with_transport(&PublicKey(4), "debug:replypath1")
      .with_transport(&PublicKey(5), "debug:replypath2"),
  ).expect("Failed to send");
  for recv in mesher.recv().expect("Failed to receive") {
    println!("received: {:?}", recv.contents());
  }
}
