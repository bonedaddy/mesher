use mesher::prelude::*;

use std::{
  io::prelude::*,
  net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
  sync::mpsc::{channel, Receiver},
  thread::Builder,
};

fn socket_addr_from_string(
  scheme: &str,
  path: String,
) -> Result<SocketAddr, TransportFail> {
  let (_, path) = path.split_at(scheme.len() + 1);
  let get_path_fail = || {
    TransportFail::InvalidURL(format!(
      "not a valid socket address format: {}",
      path
    ))
  };
  path
    .to_socket_addrs()
    .map_err(|_| get_path_fail())?
    .next()
    .ok_or(get_path_fail())
}

struct Listener {
  data: Receiver<Vec<u8>>,
}

impl Listener {
  fn new(scheme: &str, addr: SocketAddr) -> Result<Listener, TransportFail> {
    let (data_in, data_out) = channel();
    let tcp_listen = TcpListener::bind(addr).map_err(|e| {
      TransportFail::ListenFailure(format!("Failed to bind listener: {:?}", e))
    })?;

    let thread_code = move || {
      for conn in tcp_listen.incoming() {
        let mut conn = match conn {
          Ok(c) => c,
          Err(_) => continue,
        };
        let mut bytes = vec![];
        if let Err(_) = conn.read_to_end(&mut bytes) {
          continue;
        }
        if let Err(_) = data_in.send(bytes) {
          return;
        }
      }
    };

    Builder::new()
      .name(format!("TCP {}:{} listener", scheme, addr))
      .spawn(thread_code)
      .map_err(|e| {
        TransportFail::SetupFailure(format!(
          "Faield to start TCP {}: listener: {:?}",
          scheme, e
        ))
      })?;

    Ok(Listener { data: data_out })
  }
}

pub struct TCP {
  listeners: Vec<Listener>,
  scheme: String,
}

impl Transport for TCP {
  fn new(scheme: &str) -> Result<Self, TransportFail> {
    Ok(TCP {
      scheme: scheme.to_string(),
      listeners: vec![],
    })
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), TransportFail> {
    let sock = socket_addr_from_string(&self.scheme, path)?;
    let mut out = TcpStream::connect(sock).map_err(|e| {
      TransportFail::SendFailure(format!(
        "Faield to establish TCP connection: {:?}",
        e
      ))
    })?;
    out.write_all(&blob).map_err(|e| {
      TransportFail::SendFailure(format!("Failed to send data: {:?}", e))
    })?;
    Ok(())
  }

  fn listen(&mut self, path: String) -> Result<(), TransportFail> {
    let sock = socket_addr_from_string(&self.scheme, path)?;
    self.listeners.push(Listener::new(&self.scheme, sock)?);
    Ok(())
  }

  fn receive(&mut self) -> Result<Vec<Vec<u8>>, TransportFail> {
    let mut received = vec![];
    for listener in self.listeners.iter() {
      while let Ok(v) = listener.data.try_recv() {
        received.push(v);
      }
    }
    Ok(received)
  }
}
