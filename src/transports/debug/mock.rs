use std::{collections::HashMap, sync::Mutex};

lazy_static! {
  static ref PACKETS: Mutex<HashMap<String, Vec<Vec<u8>>>> = Mutex::new(HashMap::new());
}

pub struct Mock {
  listening: Vec<String>,
}

impl crate::Transport for Mock {
  fn new(_scheme: &str) -> Result<Self, crate::TransportFail> {
    Ok(Mock { listening: vec![] })
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), crate::TransportFail> {
    let mut packets = PACKETS.lock().expect("poisoned lock?");
    match packets.get_mut(&path) {
      Some(v) => v.push(blob),
      None => {
        packets.insert(path, vec![blob]);
      }
    };
    Ok(())
  }

  fn listen(&mut self, path: String) -> Result<(), crate::TransportFail> {
    self.listening.push(path);
    Ok(())
  }

  fn receive(&mut self) -> Result<Vec<Vec<u8>>, crate::TransportFail> {
    let mut packets = PACKETS.lock().expect("poisoned lock?");
    Ok(
      self
        .listening
        .iter()
        .flat_map(|path| {
          packets
            .insert(path.clone(), vec![])
            .unwrap_or(vec![])
            .into_iter()
        })
        .collect(),
    )
  }
}
