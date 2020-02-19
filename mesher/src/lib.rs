#![warn(clippy::all)]
#![doc(test(attr(deny(warnings))))]

pub mod crypto;
pub mod fail;

mod mesher;
mod packet;
mod transport;

pub use crate::{mesher::Mesher, packet::Packet, transport::Transport};

pub mod prelude {
  //! Re-exports all the commonly used parts for slightly more ergonomic use, at the expense of cluttering up the global namespace.

  pub use crate::{
    crypto::{PublicKey, SecretKey},
    fail, Mesher, Packet, Transport,
  };
}
