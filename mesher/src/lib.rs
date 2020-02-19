#![warn(clippy::all)]
#![doc(test(attr(deny(warnings))))]

// for transport::debug::InMemory
#[macro_use]
extern crate lazy_static;

pub mod crypto;
pub mod fail;
pub mod debug_transports;

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
