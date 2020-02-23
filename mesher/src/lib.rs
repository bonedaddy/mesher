#![warn(clippy::all)]
#![doc(test(attr(deny(warnings))))]

//! For information on the concepts underlying this library, see [the project repo's README](https://github.com/nic-hartley/mesher/blob/master/README.md).
//! The rest of this API documentation assumes you've read and understood it, and won't explain them.
//! The README is also available at the root of the repo, as is standard.
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
pub mod debug_transports;
pub mod fail;

mod mesher;
mod packet;
mod transport;

pub use crate::{
  mesher::{Mesher, Message},
  packet::Packet,
  transport::Transport,
};

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
