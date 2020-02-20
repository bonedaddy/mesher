use std::{collections::HashMap, sync::Mutex};

use crate::prelude::*;

lazy_static! {
  static ref PACKETS: Mutex<HashMap<String, Vec<Vec<u8>>>> = Mutex::new(HashMap::new());
}

/// A Transport implementation which "transports" data by storing and retrieving it from an in-memory store.
/// 
/// This is extremely useful when writing end-to-end tests which communicate through a mesher, if you don't want to rely on the stability of a real transport method.
/// Except in extremely rare circumstances (e.g. out-of-memory, huge numbers of messages, threads dying unexpectedly), this mesher cannot fail.
/// 
/// Always use unique paths, even across tests, as the storage used is global.
/// This is intentional, as it allows for tests which run multiple threads to simulate multiple meshers operating independently.
/// 
/// You should never use this struct directly. Instead, use it through [`Mesher`](../struct.Mesher.html), like any other transport:
/// 
/// ```no_run
/// # use mesher::prelude::*;
/// # let mut some_mesher = Mesher::unsigned(vec![]);
/// some_mesher.add_transport::<mesher::debug_transports::InMemory>("inmem")
///   .expect("Failed to add InMemory transport");
/// ```
#[allow(dead_code)]
pub struct InMemory {
  listening: Vec<String>,
}

impl Transport for InMemory {
  fn new(_scheme: &str) -> fail::Result<Self> {
    Ok(InMemory { listening: vec![] })
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> fail::Result<()> {
    let mut packets = PACKETS.lock().expect("poisoned lock?");
    match packets.get_mut(&path) {
      Some(v) => v.push(blob),
      None => {
        packets.insert(path, vec![blob]);
      }
    };
    Ok(())
  }

  fn listen(&mut self, path: String) -> fail::Result<()> {
    self.listening.push(path);
    Ok(())
  }

  fn receive(&mut self) -> fail::Result<Vec<Vec<u8>>> {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn send_and_receive() {
    let mut t = InMemory::new("inmem").expect("Failed to create");

    t.listen("inmem:1".to_owned()).expect("Failed to listen");
    t.send("inmem:1".to_owned(), vec![1, 2, 3, 4]).expect("Failed to send");
    let recvd = t.receive().expect("Failed to receive");
    assert_eq!(recvd, vec![vec![1, 2, 3, 4]]);
  }

  #[test]
  fn send_2_and_receive() {
    let mut t = InMemory::new("inmem").expect("Failed to create");

    t.listen("inmem:2".to_owned()).expect("Failed to listen");
    t.send("inmem:2".to_owned(), vec![1, 2, 3, 4]).expect("Failed to send");
    t.send("inmem:2".to_owned(), vec![5, 6, 7, 8]).expect("Failed to send");
    let recvd = t.receive().expect("Failed to receive");
    assert_eq!(recvd, vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
  }

  #[test]
  fn send_and_receive_out_of_order() {
    let mut t = InMemory::new("inmem").expect("Failed to create");

    t.send("inmem:3".to_owned(), vec![9, 10, 11, 12]).expect("Failed to send");
    t.listen("inmem:3".to_owned()).expect("Failed to listen");
    let recvd = t.receive().expect("Failed to receive");
    assert_eq!(recvd, vec![vec![9, 10, 11, 12]]);
  }

  #[test]
  fn receive_blank() {
    let mut t = InMemory::new("inmem").expect("Failed to create");

    t.listen("inmem:4".to_owned()).expect("Failed to listen");
    let recvd = t.receive().expect("Failed to receive");
    assert_eq!(recvd, Vec::<Vec<u8>>::new());
  }
}
