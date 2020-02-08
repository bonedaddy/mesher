use {crate::prelude::*, std::collections::HashMap};

#[derive(Debug)]
/// Represents a single message received by a mesher.
pub struct Message {
  contents: Vec<u8>,
}

impl Message {
  /// Get the contents of the message.
  pub fn contents(&self) -> &[u8] {
    &self.contents
  }
}

/// The control interface for a single mesher.
/// 
/// One important thing to note is that the Mesher struct **only** stores keys.
/// You will need to do responsible key management, e.g. storing them securely.
pub struct Mesher {
  transports: HashMap<String, Box<dyn Transport>>,
  own_skeys: Vec<SecretKey>,
}

impl Mesher {
  /// Creates a mesher which signs its outgoing messages with keys chosen randomly from its list.
  /// The keys are also used when receiving messages, to decrypt the ones meant for it.
  #[deprecated(note = "Not yet implemented.")]
  pub fn signed(_own_skeys: Vec<SecretKey>, _source_sigs: Vec<PublicKey>) -> Mesher {
    unimplemented!()
  }

  /// Creates a mesher which doesn't sign its outgoing messages.
  /// The keys are used when receiving messages, to decrypt the ones meant for it.
  pub fn unsigned(own_skeys: Vec<SecretKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
      own_skeys,
    }
  }

  /// Adds a transport to the mesher, for it to send and receive data through.
  /// The scheme is passed to the transport exactly as-is.
  /// If an initialization error occurs in the transport, nothing is added to the internal scheme mapping.
  pub fn add_transport<T: Transport + 'static>(&mut self, scheme: &str) -> fail::Result<()> {
    self.transports.insert(scheme.to_owned(), Box::new(T::new(scheme)?));
    Ok(())
  }

  /// Does the massaging necessary to get the transport based on the scheme in the path.
  /// Will return the appropriate errors if any of it fails.
  #[allow(clippy::borrowed_box)] // because we can't easily massage &mut Box<T> into &mut T, apparently
  fn get_transport_for_path(&mut self, path: &str) -> fail::Result<&mut Box<dyn Transport>> {
    let scheme = path
      .splitn(2, ':')
      .next()
      .ok_or_else(|| fail::MesherFail::InvalidURL("no colon-delimited scheme segment".to_string()))?
      .to_owned();
    self
      .transports
      .get_mut(&scheme)
      .ok_or(fail::MesherFail::UnregisteredScheme(scheme))
  }

  /// Has the mesher listen on the given path for messages.
  /// This determines the transport to connect to based on the scheme, then just tells it to listen.
  /// The exact behavior depends on the transport, but will generally involve either setting up some listener, or adding it to a list of internal paths to poll.
  pub fn listen_on(&mut self, path: &str) -> fail::Result<()> {
    self.get_transport_for_path(path)?.listen(path.to_owned())
  }

  /// Does everything you'd expect when mesher receives a packet:
  /// 
  /// - Attempts to decrypt every line in the packet
  /// - Forwards the packet as dictated by it
  /// - Returns any messages contained in it
  /// 
  /// It will try to use _all_ of the secret keys associated with the mesher to decrypt the packet.
  fn process_packet(&mut self, pkt: Vec<u8>) -> fail::Result<Vec<Message>> {
    let dis = crate::packet::Packet::from_bytes(&pkt, &self.own_skeys)?;
    let mut messages = vec![];
    for piece in dis {
      match piece {
        crate::packet::Chunk::Message(m) => messages.push(Message { contents: m }),
        crate::packet::Chunk::Transport(to) => self.bounce(&pkt, &to)?,
        crate::packet::Chunk::Encrypted(_) => (), /* piece not meant for us */
      }
    }
    Ok(messages)
  }

  /// Sends a packet out.
  /// Note that the packet is not processed, so any instructions meant for this mesher will not be seen (unless the packet comes back, of course)
  pub fn launch(&mut self, packet: crate::packet::Packet, first_hop: &str) -> fail::Result<()> {
    self.bounce(&packet.into_bytes()?, first_hop)
  }

  // Sends the given bytes along the given path, getting the appropriate transport.
  fn bounce(&mut self, packet: &[u8], path: &str) -> fail::Result<()> {
    let transport = self.get_transport_for_path(path)?;
    transport.send(path.to_owned(), packet.to_vec())?;
    Ok(())
  }

  /// Gets pending messages from all of the transports along all of the paths they've been told to use.
  pub fn recv(&mut self) -> fail::Result<Vec<Message>> {
    // don't focus too much on how I got this...
    let mut packets = vec![];
    for (_, transport) in self.transports.iter_mut() {
      packets.append(&mut transport.receive()?);
    }
    let mut messages = vec![];
    for p in packets {
      messages.append(&mut self.process_packet(p)?);
    }
    Ok(messages)
  }
}
