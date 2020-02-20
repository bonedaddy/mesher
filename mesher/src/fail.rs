// TODO: don't use String error messages, pass relevant data

//! Contains the error-reporting enum for mesher.

/// Every possible way a [`Mesher`](../struct.Mesher.html) can fail to do something.
///
/// Generally split into two categories, mesher and transport errors.
/// The transports can error out in any stage of their lifetime except dropping: setup, sending, listening, or receiving.
///
/// This enum is `#[non_exhaustive]` because future releases are all but guaranteed to add more specific, and therefore more helpful, error states.
#[non_exhaustive]
#[derive(Debug)]
pub enum MesherFail {
  /// The mesher needed at least one secret key but none were available.
  ///
  /// This is only triggerd when keys are actually used, *not* during initialization.
  /// Meshers can generally be safely created without keys, then have them added, so long as it doesn't try to receive messages or send signed ones.
  NoKeys,

  /// A mesher received a packet in a format that couldn't be parsed into a packet.
  ///
  /// Note that packets with no chunks encrypted for the receiving mesher will not be treated as an error.
  /// They will be no-ops.
  /// This error means that the packet itself had an invalid structure.
  InvalidPacket,

  /// The URL passed as the path to transport a packet along is invalid.
  InvalidURL(String),
  /// The URL's scheme hasn't been registered with the mesher, so it can't know what transport to use to move the packet.
  UnregisteredScheme(String),

  /// The transport being asked to listen on a path wasn't able to.
  SetupFailure(String),

  /// The transport being asked to send data along a path wasn't able to.
  ///
  /// This can trigger during calls to [`Mesher::recv`](../struct.Mesher.html), since it will send packets as requested while parsing them.
  SendFailure(String),

  /// The transport being asked to listen along a path wasn't able to.
  ListenFailure(String),

  /// The transport being asked to fetch all received messages wasn't able to.
  ReceiveFailure(String),

  /// Some other error happened.
  /// Ideally, this would never be returned, but it's left as an option just in case, or for debugging.
  Other(Box<dyn std::error::Error>),
}

/// A `Result` alias with [`MesherFail`](enum.MesherFail.html) as the Err type to make some code a little less repetitive.
pub type Result<TOk> = std::result::Result<TOk, MesherFail>;
