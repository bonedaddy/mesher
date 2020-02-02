use std::{collections::HashMap, sync::Mutex};

use mesher::prelude::*;

lazy_static! {
  static ref PACKETS: Mutex<HashMap<String, Vec<Vec<u8>>>> = Mutex::new(HashMap::new());
}

#[allow(dead_code)]
pub struct InMemory {
  listening: Vec<String>,
}

impl Transport for InMemory {
  fn new(_scheme: &str) -> Result<Self, TransportFail> {
    Ok(InMemory { listening: vec![] })
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), TransportFail> {
    let mut packets = PACKETS.lock().expect("poisoned lock?");
    match packets.get_mut(&path) {
      Some(v) => v.push(blob),
      None => {
        packets.insert(path, vec![blob]);
      }
    };
    Ok(())
  }

  fn listen(&mut self, path: String) -> Result<(), TransportFail> {
    self.listening.push(path);
    Ok(())
  }

  fn receive(&mut self) -> Result<Vec<Vec<u8>>, TransportFail> {
    let mut packets = PACKETS.lock().expect("poisoned lock?");
    Ok(
      self
        .listening
        .iter()
        .flat_map(|path| {
          packets
            .insert(path.clone(), vec![])
            .unwrap_or_else(|| vec![])
            .into_iter()
        })
        .collect(),
    )
  }
}
