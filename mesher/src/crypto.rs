extern crate sodiumoxide;

pub mod encrypt {
  pub use sodiumoxide::crypto::box_::{gen_keypair, PublicKey, SecretKey};

  pub(crate) use sodiumoxide::crypto::{
    sealedbox::seal,
  };

  pub(crate) fn open(c: &[u8], key: &SecretKey) -> Result<Vec<u8>, ()> {
    sodiumoxide::crypto::sealedbox::open(c, &key.public_key(), key)
  }
}

pub mod sign {
  pub use sodiumoxide::crypto::sign::{
    PublicKey,
    SecretKey,
    gen_keypair,
  };
  
  pub(crate) use sodiumoxide::crypto::sign::{
    sign,
    verify,
  };
}
