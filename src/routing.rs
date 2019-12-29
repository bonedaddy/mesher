pub struct Route; /* {
                    target_key: crate::PublicKey,
                    transports: Vec<(crate::PublicKey, String)>,
                    reply_route: Option<Box<Route>>,
                  } */
impl Route {
  pub fn to(_target_key: crate::PublicKey) -> Route {
    Route
  }
  pub fn with_transport(&mut self, _node_key: &crate::PublicKey, _transport: &str) -> &mut Route {
    self
  }
  pub fn reply_to(&mut self, _node_key: crate::PublicKey) -> &mut Route {
    self
  }
}
