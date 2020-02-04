#![warn(clippy::all)]

#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "debug")]
#[macro_use]
extern crate lazy_static;

pub mod crypto;
pub mod packet;
pub mod transports;
pub mod mesher;
pub mod fail;

pub mod prelude {
  pub use crate::{
    crypto::{PublicKey, SecretKey},
    transports::Transport,
    fail::MesherFail,
    mesher::Mesher,
    packet::SimpleRoute,
  };
}
