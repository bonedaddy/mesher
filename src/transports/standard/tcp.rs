pub struct TCP {}

impl crate::transports::Transport for TCP {
  fn new(scheme: &str) -> Result<Self, crate::transports::TransportFail> {
    Ok(TCP {})
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), crate::transports::TransportFail> {
    Ok(())
  }

  fn listen(&mut self, path: String) -> Result<(), crate::transports::TransportFail> {
    Ok(())
  }

  fn receive(&mut self) -> Result<Vec<Vec<u8>>, crate::transports::TransportFail> {
    Ok(vec![])
  }
}
