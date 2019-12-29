use std::{error, fmt};
use itertools::Itertools;

#[derive(fmt::Debug)]
pub struct DebugFail;
impl fmt::Display for DebugFail {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "The debug transport somehow failed??")
  }
}
impl error::Error for DebugFail {}

pub struct Debug {
  prefix: String,
}

impl super::Transport for Debug {
  type Fail = DebugFail;
  
  fn new(prefix: &str) -> Result<Self, Self::Fail> {
    println!("Creating debug transport under prefix {}", prefix);
    Ok(Debug { prefix: prefix.to_owned() })
  }

  fn send(&mut self, blob: Vec<u8>) -> Result<(), Self::Fail> {
    println!("Debug {} sent {} bytes. In hex:", self.prefix, blob.len());
    // most of the message
    for s in 0..blob.len() / 16 {
      let bytes = blob[s..s + 16].iter().map(|b| format!("{:x}", b)).join("");
      println!("{:4x}: {}", s, bytes);
  }
  // last line
  let overhang = blob.len() % 16;
  if overhang > 0 {
      let oh_start = blob.len() - overhang;
      let bytes = blob[oh_start..].iter().map(|b| format!("{:02x}", b)).join("");
      println!("{:4x}: {}", oh_start, bytes);
  }
  Ok(())
  }

  fn recv(&mut self) -> Result<Vec<Vec<u8>>, Self::Fail> {
    Ok(vec![])
  }
}
