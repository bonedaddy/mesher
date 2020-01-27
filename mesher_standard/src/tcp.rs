use mesher::prelude::*;

pub struct TCP {

}

impl Transport for TCP {
  fn new(scheme: &str) -> Result<Self, TransportFail> {
    Ok(TCP {})
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), TransportFail> {
    Ok(())
  }

  fn listen(&mut self, path: String) -> Result<(), TransportFail> {
    Ok(())
  }

  fn receive(&mut self) -> Result<Vec<Vec<u8>>, TransportFail> {
    Ok(vec![])
  }
}
