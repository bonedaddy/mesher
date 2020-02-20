#![warn(clippy::all)]
#![doc(test(attr(deny(warnings))))]

//! Mesher is a fairly simple wrapper over a fairly complex concept:
//! Securely and anonymously sending a message over a heterogeneously connected mesh network.
//! 
//! In smaller, more real words, mesher:
//! 
//! - Operates on mesh networks
//! - Where nodes may be connected through a variety of channels (heterogeneous, not homogeneous)
//! - Prevents intermediate nodes from knowing the source or destination
//! 
//! It's primarily desigend for anonymous, one-way communication

// for transport::debug::InMemory
#[macro_use]
extern crate lazy_static;

pub mod crypto;
pub mod fail;
pub mod debug_transports;

mod mesher;
mod packet;
mod transport;

pub use crate::{mesher::{Mesher, Message}, packet::Packet, transport::Transport};

pub mod prelude {
  //! Re-exports all the commonly used parts for slightly more ergonomic use, at the expense of cluttering up the global namespace.
  //! Usage:
  //! 
  //! ```
  //! # #[allow(unused_imports)]
  //! use mesher::prelude::*;
  //! ```

  pub use crate::{
    crypto::{PublicKey, SecretKey},
    fail, Mesher, Message, Packet, Transport,
  };
}
