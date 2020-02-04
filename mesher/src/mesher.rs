use {
  std::collections::HashMap,
  crate::prelude::*,
};

#[derive(Debug)]
pub struct Message {
  contents: Vec<u8>,
}

impl Message {
  pub fn contents(&self) -> &[u8] {
    &self.contents
  }
}

pub struct Mesher {
  transports: HashMap<String, Box<dyn Transport>>,
  own_skeys: Vec<SecretKey>,
}

impl Mesher {
  pub fn signed(own_skeys: Vec<SecretKey>, _source_sigs: Vec<PublicKey>) -> Mesher {
    // TODO: outgoing packet signature setup
    Mesher::unsigned(own_skeys)
  }
  pub fn unsigned(own_skeys: Vec<SecretKey>) -> Mesher {
    Mesher {
      transports: HashMap::new(),
      own_skeys,
    }
  }

  pub fn add_transport<T: Transport + 'static>(&mut self, scheme: &str) -> Result<(), MesherFail> {
    self.transports.insert(scheme.to_owned(), Box::new(T::new(scheme)?));
    Ok(())
  }

  #[allow(clippy::borrowed_box)]
  fn get_transport_for_path(&mut self, path: &str) -> Result<&mut Box<dyn Transport>, MesherFail> {
    let scheme = path
      .splitn(2, ':')
      .next()
      .ok_or_else(|| MesherFail::InvalidURL("no colon-delimited scheme segment".to_string()))?
      .to_owned();
    self
      .transports
      .get_mut(&scheme)
      .ok_or(MesherFail::UnregisteredScheme(scheme))
  }

  pub fn listen_on(&mut self, path: &str) -> Result<(), MesherFail> {
    self.get_transport_for_path(path)?.listen(path.to_owned())
  }

  fn process_packet(&mut self, pkt: Vec<u8>) -> crate::fail::Result<Vec<Message>> {
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

  pub fn send(&mut self, message: &[u8], route: crate::packet::SimpleRoute) -> crate::fail::Result<()> {
    let (packet, hop) = crate::packet::Packet::along_route(message, route);
    self.launch(packet, &hop)
  }

  pub fn launch(&mut self, packet: crate::packet::Packet, first_hop: &str) -> crate::fail::Result<()> {
    self.bounce(&packet.into_bytes()?, first_hop)
  }

  fn bounce(&mut self, packet: &[u8], path: &str) -> crate::fail::Result<()> {
    let transport = self.get_transport_for_path(path)?;
    transport.send(path.to_owned(), packet.to_vec())?;
    Ok(())
  }

  pub fn recv(&mut self) -> crate::fail::Result<Vec<Message>> {
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
