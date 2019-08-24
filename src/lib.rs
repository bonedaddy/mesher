extern crate bincode;

extern crate serde;

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as nacl;

mod crypto;
use crypto::*;

// TODO: C api

#[derive(Debug)]
pub enum Error {
    BincodeFail(bincode::Error),
    CryptoFail,
    #[cfg(feature = "handling")]
    SendFailed(String),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Shell(String, nacl::PublicKey),
    ShellOutput(Vec<u8>),
    Forward(String),
    Print(String), // for testing purposes
}



pub enum Permission {
    ForwardText,
}

pub struct Packet {

}

pub fn serialize_packet(
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
    bytes: &[u8],
    sender_pkey: &nacl::PublicKey,
    recver_skey: &nacl::SecretKey,
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
