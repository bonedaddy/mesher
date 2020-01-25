#[derive(Debug)]
#[non_exhaustive]
pub enum Fail {
  NoKeys,
  NoReplyRoute,

  NotYetImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, Fail>;
