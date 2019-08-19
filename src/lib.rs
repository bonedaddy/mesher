extern crate bincode;

extern crate serde;

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as crypto;

// TODO: C api

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Forward(String),
    Custom(Vec<u8>),
}

#[derive(Debug)]
pub enum Error {
    SerializeFail(bincode::Error),
    CryptoFail,
    #[cfg(feature = "handling")]
    MessageSendFailed,
}
pub type Result<T> = std::result::Result<T, Error>;

pub fn pack_commands(
    sender_skey: &crypto::SecretKey,
    cmds: &[(Command, &crypto::PublicKey)],
) -> Result<Vec<u8>> {
    sodiumoxide::init().map_err(|_| Error::CryptoFail)?;

    let mut elems: Vec<Vec<u8>> = Vec::new();
    for cmd in cmds {
        let serd = bincode::serialize(&cmd.0).map_err(Error::SerializeFail)?;
        let nonce = crypto::gen_nonce();
        let sealed = crypto::seal(&serd, &nonce, cmd.1, sender_skey);
        let elem = nonce.0.iter().cloned().chain(sealed.into_iter()).collect();
        elems.push(elem);
    }
    bincode::serialize(&elems).map_err(Error::SerializeFail)
}

fn open_elem(
    elem: Vec<u8>,
    pkey: &crypto::PublicKey,
    skey: &crypto::SecretKey,
) -> Option<Command> {
    if elem.len() < crypto::NONCEBYTES {
        return None;
    }
    let (nonce, elem) = elem.split_at(crypto::NONCEBYTES);
    let nonce = crypto::Nonce::from_slice(nonce)?;
    let opened = crypto::open(elem, &nonce, pkey, skey).ok()?;
    bincode::deserialize(&opened).ok()
}

pub fn unpack_commands(
    sender_pkey: &crypto::PublicKey,
    receiver_skey: &crypto::SecretKey,
    bytes: &[u8],
) -> Result<Vec<Command>> {
    let ded = bincode::deserialize::<Vec<Vec<u8>>>(&bytes)
            .map_err(Error::SerializeFail)?;
    Ok(ded.into_iter().filter_map(|el| open_elem(el, sender_pkey, receiver_skey)).collect())
}

#[cfg(feature = "handling")]
pub mod handle;
