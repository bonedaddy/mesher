extern crate bincode;

extern crate serde;

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as crypto;

use crate::{Error, Result};

pub fn encrypt(
    clear: &[u8],
    sender_skey: &crypto::SecretKey,
    recver_pkey: &crypto::PublicKey,
) -> Result<Vec<u8>> {
    let nonce = crypto::gen_nonce();
    let sealed = crypto::seal(clear, &nonce, recver_pkey, sender_skey);
    Ok(nonce
        .0
        .to_vec()
        .into_iter()
        .chain(sealed.into_iter())
        .collect())
}

pub fn ser_encrypt<T: serde::Serialize>(
    clear: &T,
    sender_skey: &crypto::SecretKey,
    recver_pkey: &crypto::PublicKey,
) -> Result<Vec<u8>> {
    let serd = bincode::serialize(clear).map_err(Error::BincodeFail)?;
    encrypt(&serd, sender_skey, recver_pkey)
}

pub fn decrypt(
    cipher: &[u8],
    sender_pkey: &crypto::PublicKey,
    recver_skey: &crypto::SecretKey,
) -> Result<Vec<u8>> {
    if cipher.len() < crypto::NONCEBYTES {
        return Err(crate::Error::CryptoFail);
    }
    let (nonceb, cipher) = cipher.split_at(crypto::NONCEBYTES);
    let nonce = crypto::Nonce::from_slice(nonceb).ok_or(Error::CryptoFail)?;
    crypto::open(cipher, &nonce, &sender_pkey, &recver_skey)
        .map_err(|_| Error::CryptoFail)
}

pub fn de_decrypt<T: serde::de::DeserializeOwned>(
    cipher: &[u8],
    sender_pkey: &crypto::PublicKey,
    recver_skey: &crypto::SecretKey,
) -> Result<T> {
    let decrypted = decrypt(cipher, sender_pkey, recver_skey)?;
    bincode::deserialize(&decrypted).map_err(Error::BincodeFail)
}
