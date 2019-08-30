extern crate bincode;

extern crate serde;

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as nacl;

use crate::{Error, Result};

pub use sodiumoxide::init as init;
pub use nacl::PublicKey as PublicKey;
pub use nacl::SecretKey as SecretKey;
pub use nacl::Nonce as Nonce;

pub fn encrypt(
    clear: &[u8],
    sender_skey: &nacl::SecretKey,
    recver_pkey: &nacl::PublicKey,
) -> Result<Vec<u8>> {
    let nonce = nacl::gen_nonce();
    let sealed = nacl::seal(clear, &nonce, recver_pkey, sender_skey);
    Ok(nonce
        .0
        .to_vec()
        .into_iter()
        .chain(sealed.into_iter())
        .collect())
}

pub fn ser_encrypt<T: serde::Serialize>(
    clear: &T,
    sender_skey: &nacl::SecretKey,
    recver_pkey: &nacl::PublicKey,
) -> Result<Vec<u8>> {
    let serd = bincode::serialize(clear).map_err(Error::BincodeFail)?;
    encrypt(&serd, sender_skey, recver_pkey)
}

pub fn decrypt(
    cipher: &[u8],
    sender_pkey: &nacl::PublicKey,
    recver_skey: &nacl::SecretKey,
) -> Result<Vec<u8>> {
    if cipher.len() < nacl::NONCEBYTES {
        return Err(crate::Error::CryptoFail);
    }
    let (nonceb, cipher) = cipher.split_at(nacl::NONCEBYTES);
    let nonce = nacl::Nonce::from_slice(nonceb).ok_or(Error::CryptoFail)?;
    nacl::open(cipher, &nonce, &sender_pkey, &recver_skey)
        .map_err(|_| Error::CryptoFail)
}

pub fn de_decrypt<T: serde::de::DeserializeOwned>(
    cipher: &[u8],
    sender_pkey: &nacl::PublicKey,
    recver_skey: &nacl::SecretKey,
) -> Result<T> {
    let decrypted = decrypt(cipher, sender_pkey, recver_skey)?;
    bincode::deserialize(&decrypted).map_err(Error::BincodeFail)
}
