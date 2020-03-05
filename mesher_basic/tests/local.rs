use mesher::prelude::*;
use mesher_basic::TCP;

use std::{thread::sleep, time::Duration};

fn make_mesher(port: Option<u16>) -> (Mesher, encrypt::PublicKey) {
  let (pk, sk) = encrypt::gen_keypair();
  let mut m = Mesher::unsigned(vec![sk]);
  m.add_transport::<TCP>("tcp").expect("Failed to add transport");
  if let Some(port) = port {
    m.listen_on(&format!("tcp:localhost:{}", port)).expect("Failed to listen");
  }
  (m, pk)
}

#[test]
fn direct() {
  let (mut m_source, _) = make_mesher(None);
  let (mut m_dest, k_dest) = make_mesher(Some(18540));

  let mut packet = Packet::unsigned();
  packet.add_message(&[1, 2, 3], &k_dest);
  m_source.launch(packet, "tcp:localhost:18540").expect("Failed to send");
  println!("Message sent");

  sleep(Duration::from_millis(100));

  let recvd = m_dest.recv().expect("failed to recv").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();
  assert_eq!(vec![vec![1, 2, 3]], recvd);
}

#[test]
fn one_hop() {
  let (mut m_source, _) = make_mesher(None);
  let (mut m_bounce, k_bounce) = make_mesher(Some(18550));
  let (mut m_dest, k_dest) = make_mesher(Some(18551));

  let mut packet = Packet::unsigned();
  packet.add_message(&[1, 2, 3], &k_dest);
  packet.add_hop("tcp:localhost:18551".to_owned(), &k_bounce);
  m_source.launch(packet, "tcp:localhost:18550").expect("Failed to send");
  println!("Message sent");

  sleep(Duration::from_millis(100));
  m_bounce.recv().expect("failed to bounce");
  sleep(Duration::from_millis(100));

  let recvd = m_dest.recv().expect("failed to recv").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();
  assert_eq!(vec![vec![1, 2, 3]], recvd);
}

#[test]
fn two_hops() {
  let (mut m_source, _) = make_mesher(None);
  let (mut m_bounce1, k_bounce1) = make_mesher(Some(18560));
  let (mut m_bounce2, k_bounce2) = make_mesher(Some(18561));
  let (mut m_dest, k_dest) = make_mesher(Some(18562));

  let mut packet = Packet::unsigned();
  packet.add_message(&[1, 2, 3], &k_dest);
  packet.add_hop("tcp:localhost:18561".to_owned(), &k_bounce1);
  packet.add_hop("tcp:localhost:18562".to_owned(), &k_bounce2);
  m_source.launch(packet, "tcp:localhost:18560").expect("Failed to send");
  println!("Message sent");

  sleep(Duration::from_millis(100));
  m_bounce1.recv().expect("failed to bounce");
  sleep(Duration::from_millis(100));
  m_bounce2.recv().expect("failed to bounce");
  sleep(Duration::from_millis(100));

  let recvd = m_dest.recv().expect("failed to recv").into_iter().map(|m| m.into_contents()).collect::<Vec<_>>();
  assert_eq!(vec![vec![1, 2, 3]], recvd);
}
