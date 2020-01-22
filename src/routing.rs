#[derive(Debug)]
pub struct Route;

impl Route {
  pub fn to(_target_key: crate::PublicKey) -> Route {
    Route
  }
  pub fn with_transport(self, _node_key: &crate::PublicKey, _transport: &str) -> Route {
    self
  }
  pub fn reply_to(self, _node_key: crate::PublicKey) -> Route {
    self
  }
}
