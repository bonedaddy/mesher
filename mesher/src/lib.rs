#![warn(clippy::all)]

#[cfg(feature = "debug")]
#[macro_use]
extern crate lazy_static;

mod crypto;
mod fail;
mod packet;
mod transports;
mod mesher;

pub mod prelude {
  pub use crate::{
    crypto::{PublicKey, SecretKey},
    transports::{Transport, TransportFail},
    mesher::Mesher, packet::SimpleRoute,
  };
}
