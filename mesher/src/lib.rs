#![warn(clippy::all)]
#![doc(test(attr(deny(warnings))))]

pub mod crypto;
pub mod fail;

mod mesher;
mod packet;
mod transport;

pub use crate::{
  mesher::Mesher,
  transport::Transport,
  packet::Packet,
};

pub mod prelude {
  pub use crate::{
    crypto::{PublicKey, SecretKey},
    fail::MesherFail,
    Mesher,
    Transport,
    Packet,
  };
}
