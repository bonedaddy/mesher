// TODO Merge this and transports::TransportFail?

use crate::prelude::*;

#[derive(Debug)]
#[non_exhaustive]
pub enum Fail {
  NoKeys,
  NoReplyRoute,

  NotYetImplemented(&'static str),

  TransportFail(TransportFail),
}

impl From<TransportFail> for Fail {
  fn from(t: TransportFail) -> Fail {
    Fail::TransportFail(t)
  }
}

pub type Result<T> = std::result::Result<T, Fail>;
