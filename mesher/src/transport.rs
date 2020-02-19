use crate::prelude::*;

/// Transport is the core of mesher's communication system.
///
/// All the ways that mesher can communicate are defined through this interface.
/// It's deliberately left vague -- mesher doesn't care how the bytes are transported, and communication channels shouldn't care what bytes they're transporting.
/// This ensures that typical transports can be reused across multiple versions of mesher without changes.
/// It also ensures that transports can be largely reused for other projects which want to communicate over those methods.
pub trait Transport {
  /// Creates a new instance of this transport method, associated with the given scheme.
  /// This isn't meant to be called by the end user; it's used by mesher internally.
  /// It should perform as little error-prone work as possible, and what errors happen should be fixable (possibly just by waiting and retrying) to the greatest extent possible.
  fn new(scheme: &str) -> fail::Result<Self>
  where
    Self: Sized;

  /// Sends some bytes through this transport method.
  /// The transport should *not* care about the bytes being sent, only (possibly) the quantity.
  /// The path will include the `scheme:` prefix.
  fn send(&mut self, path: String, blob: Vec<u8>) -> fail::Result<()>;

  /// Set up this transport to listen on the given path.
  /// This does not return any messages -- it just tells the transport to listen on/poll on this route to receive future messages.
  /// The path will include the `scheme:` prefix.
  fn listen(&mut self, path: String) -> fail::Result<()>;

  /// Actually receive the pending messages.
  /// In listen-based transports, this will simply pull the received messages from the listener.
  /// In poll-based ones, it will actually perform the poll.
  /// The paths to receive on are given through calls to [`Transport::listen`][1]
  ///
  ///  [1]: #tymethod.listen
  fn receive(&mut self) -> fail::Result<Vec<Vec<u8>>>;
}

mod debug {

}