#![warn(clippy::all)]
#![doc(test(attr(deny(warnings))))]

//! Mesher is a fairly simple wrapper over a fairly complex concept:
//! Securely and anonymously sending a message over a heterogeneously connected mesh network.
//! 
//! In smaller, more real words, mesher:
//! 
//! - Operates on mesh networks
//! - Allows nodes to be connected through a variety of channels (heterogeneous, not homogeneous)
//! - Prevents intermediate nodes from knowing the source or destination
//! - Prevents intermediate nodes from knowing the contents of the message
//! 
//! It's primarily desigend for anonymous, one-way communication.
//! However, replies facilitate round-trip communications, which in turn can be used to make a tunnel.
//! 
//! For more information on the concepts underlying this library, see [the project repo's README](https://github.com/nic-hartley/mesher/blob/master/README.md).
//! The rest of this API documentation assumes you've read and understood it.
//! 
//! The mesher API is fairly simple, and based around three pieces, which reflect the concepts described in the README:
//! 
//! - [`struct Mesher`](struct.Mesher.html) coordinates the rest of the objects, e.g. managing Transports, automatically handling bounces, etc.
//! - [`trait Transport`](trait.Transport.html) defines the interface that `Mesher` uses to control Transports.
//!   If you need them, e.g. for testing, there are debug transports available in [`mesher::debug_transports`](debug_transports/index.html).
//! - [`struct Packet`](struct.Packet.html) makes building signed and unsigned packets easier.
//! 
//! Also worth mentioning are the types in [`mesher::crypto`](crypto/index.html), which encapsulate the manipulation of crypto primitives.
//! You'll use them to pass keys into `Mesher` and `Packet`.
//! They do offer secure keygen, but this crate **will not** handle storing keys for you, if you need that.
//! 
//! [`struct Message`](struct.Message.html) represents a message received.
//! 
//! There is, of course, a [`fail`](fail/index.html) module, with the expected [`enum MesherFail`](fail/enum.MesherFail.html) and [`type Result`](fail/type.Result.html) for this crate's error handling.

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
