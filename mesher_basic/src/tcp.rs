use mesher::prelude::*;

use std::{
  io::prelude::*,
  net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
  sync::mpsc::{channel, Receiver, Sender},
  thread::Builder,
};

fn socket_addr_from_string(scheme: &str, path: String) -> fail::Result<SocketAddr> {
  let (_, path) = path.split_at(scheme.len() + 1);
  let get_path_fail = || fail::MesherFail::InvalidURL(format!("not a valid socket address format: {}", path));
  path
    .to_socket_addrs()
    .map_err(|_| get_path_fail())?
    .next()
    .ok_or_else(get_path_fail)
}

fn listen(scheme: &str, addr: SocketAddr, sender: Sender<Vec<u8>>) -> fail::Result<()> {
  let tcp_listen = TcpListener::bind(addr)
    .map_err(|e| fail::MesherFail::ListenFailure(format!("Failed to bind listener: {:?}", e)))?;

  let thread_code = move || {
    for conn in tcp_listen.incoming() {
      let mut conn = match conn {
        Ok(c) => c,
        Err(_) => continue,
      };
      let mut bytes = vec![];
      if conn.read_to_end(&mut bytes).is_err() {
        continue;
      }
      if sender.send(bytes).is_err() {
        return;
      }
    }
  };

  Builder::new()
    .name(format!("TCP {}:{} listener", scheme, addr))
    .spawn(thread_code)
    .map_err(|e| fail::MesherFail::SetupFailure(format!("Faield to start TCP {}: listener: {:?}", scheme, e)))?;

  Ok(())
}

pub struct TCP {
  sender: Sender<Vec<u8>>,
  receiver: Receiver<Vec<u8>>,
  scheme: String,
}

impl Transport for TCP {
  fn new(scheme: &str) -> fail::Result<Self> {
    let (sender, receiver) = channel();
    Ok(TCP {
      scheme: scheme.to_string(),
      sender,
      receiver,
    })
  }

  fn send(&mut self, path: String, blob: Vec<u8>) -> fail::Result<()> {
    let sock = socket_addr_from_string(&self.scheme, path)?;
    let mut out = TcpStream::connect(sock)
      .map_err(|e| fail::MesherFail::SendFailure(format!("Failed to establish TCP connection: {:?}", e)))?;
    out
      .write_all(&blob)
      .map_err(|e| fail::MesherFail::SendFailure(format!("Failed to send data: {:?}", e)))?;
    Ok(())
  }

  fn listen(&mut self, path: String) -> fail::Result<()> {
    let sock = socket_addr_from_string(&self.scheme, path)?;
    listen(&self.scheme, sock, self.sender.clone())?;
    Ok(())
  }

  fn receive(&mut self) -> fail::Result<Vec<Vec<u8>>> {
    Ok(self.receiver.try_iter().collect())
  }
}
