use crate::prelude::*;

pub trait Transport {
  fn new(scheme: &str) -> Result<Self, MesherFail>
  where
    Self: Sized;
  fn send(&mut self, path: String, blob: Vec<u8>) -> Result<(), MesherFail>;
  fn listen(&mut self, path: String) -> Result<(), MesherFail>;
  fn receive(&mut self) -> Result<Vec<Vec<u8>>, MesherFail>;
}
