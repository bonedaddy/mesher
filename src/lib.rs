extern crate bincode;

extern crate serde;

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as nacl;

mod crypto;
use crypto::*;

// TODO: C api

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Forward(String),
    Custom(Vec<u8>),
}

#[derive(Debug)]
pub enum Error {
    BincodeFail(bincode::Error),
    CryptoFail,
    #[cfg(feature = "handling")]
    MessageSendFailed,
}
pub type Result<T> = std::result::Result<T, Error>;

pub fn pack_commands(
    sender_skey: &nacl::SecretKey,
    cmds: &[(Command, &nacl::PublicKey)],
) -> Result<Vec<u8>> {
    sodiumoxide::init().map_err(|_| Error::CryptoFail)?;

    let mut elems: Vec<Vec<u8>> = Vec::new();
    for cmd in cmds {
        elems.push(ser_encrypt(&cmd.0, sender_skey, cmd.1)?);
    }
    bincode::serialize(&elems).map_err(Error::BincodeFail)
}

pub fn unpack_commands(
    sender_pkey: &nacl::PublicKey,
    recver_skey: &nacl::SecretKey,
    bytes: &[u8],
) -> Result<Vec<Command>> {
    let ded: Vec<Vec<u8>> =
        bincode::deserialize(&bytes).map_err(Error::BincodeFail)?;
    Ok(ded
        .into_iter()
        .filter_map(|el| de_decrypt(&el, sender_pkey, recver_skey).ok())
        .collect())
}

#[cfg(feature = "handling")]
pub mod handle;
