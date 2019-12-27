#![warn(clippy::all)]

use x25519_dalek::PublicKey as PublicKey;
use x25519_dalek::StaticSecret as SecretKey;

pub mod fail;
pub mod routing;
pub use routing::Route;
pub mod transports;
pub use transports::Transport;

pub struct Message {
  pub contents: Vec<u8>,
  pub reply_route: Option<Route>,
}

pub struct Mesher; /* {
  own_keys: Vec<crate::SecretKey>,
  source_sigs: Vec<crate::PublicKey>,
  transports: HashMap<String, Box<dyn Transport>>,
} */

impl Mesher {
  pub fn new(_source_sigs: Vec<crate::PublicKey>) -> Mesher { Mesher }
  pub fn add_own_key(&mut self, _k: crate::SecretKey) {}
  pub fn add_sender_key(&mut self, _k: crate::PublicKey) {}

  pub fn send(&mut self, _message: &[u8], _route: Route) -> fail::Result<()> { Ok(()) }
  pub fn reply(&mut self, _message: &[u8], _to: Message) -> fail::Result<()> { Ok(()) }

  pub fn recv(&mut self) -> fail::Result<Vec<Message>> { Ok(vec![]) }
}
