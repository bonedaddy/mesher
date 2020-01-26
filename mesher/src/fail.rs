// TODO Merge this and transports::TransportFail?

#[derive(Debug)]
#[non_exhaustive]
pub enum Fail {
  NoKeys,
  NoReplyRoute,

  NotYetImplemented(&'static str),

  TransportFail(crate::TransportFail),
}

impl From<crate::TransportFail> for Fail {
  fn from(t: crate::TransportFail) -> Fail {
    Fail::TransportFail(t)
  }
}

pub type Result<T> = std::result::Result<T, Fail>;
