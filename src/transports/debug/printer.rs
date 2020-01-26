use itertools::Itertools;

pub struct Printer {
  scheme: String,
}

impl crate::Transport for Printer {
  fn new(scheme: &str) -> Result<Self, crate::TransportFail> {
    if !scheme.starts_with('d') {
      return Err(crate::TransportFail::UnsupportedScheme("must start with d"));
    }
    println!("Creating Printer transport under scheme {}", scheme);
    Ok(Printer {
      scheme: scheme.to_owned(),
    })
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), crate::TransportFail> {
    println!(
      "Printer {} sent {} bytes to {}. In hex:",
      self.scheme,
      path,
      blob.len()
    );
    // most of the message
    for s in 0..blob.len() / 16 {
      let bytes = blob[s..s + 16].iter().map(|b| format!("{:x}", b)).join("");
      println!("{:4x}: {}", s, bytes);
    }
    // last line
    let overhang = blob.len() % 16;
    if overhang > 0 {
      let oh_start = blob.len() - overhang;
      let bytes = blob[oh_start..]
        .iter()
        .map(|b| format!("{:02x}", b))
        .join("");
      println!("{:4x}: {}", oh_start, bytes);
    }
    Ok(())
  }

  fn recv(&mut self) -> Result<Vec<Vec<u8>>, crate::TransportFail> {
    println!("Printer {} polled for more data", self.scheme);
    Ok(vec![])
  }
}
