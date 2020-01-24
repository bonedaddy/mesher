#[derive(Debug)]
pub struct Route {
  target: crate::PublicKey,
  first_hop: String,
  transports: Vec<(String, crate::PublicKey)>,
  reply: Option<Box<Route>>,
}

impl Route {
  pub fn to(target_key: crate::PublicKey, first_hop: &str) -> Route {
    println!("Creating route to {:?}", target_key);
    Route {
      target: target_key,
      first_hop: first_hop.to_owned(),
      transports: Vec::new(),
      reply: None,
    }
  }
  pub fn with_transport(mut self, node_key: &crate::PublicKey, transport: &str) -> Route {
    println!("Adding transport {} for node {:?}", transport, node_key);
    self
      .transports
      .push((transport.to_owned(), node_key.clone()));
    self
  }
  pub fn reply_to(mut self, path: Route) -> Route {
    println!("Directing replies along {:?}", path);
    self.reply = Some(Box::new(path));
    self
  }
}
