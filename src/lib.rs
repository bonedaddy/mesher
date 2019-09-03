extern crate bincode;

extern crate serde;

extern crate sodiumoxide;

#[macro_use]
extern crate bitflags;

mod crypto;

// TODO: C api

#[derive(Debug)]
pub enum Error {
    BincodeFail(bincode::Error),
    CryptoFail(crypto::Error),
    #[cfg(feature = "handling")]
    SendFailed(String),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Shell(String, crypto::PublicKey),
    ShellOutput(Vec<u8>),
    Forward(String),
    Print(String), // for testing purposes
}

bitflags! {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Allowance: u8 {
        const FORWARD   = 0x01;
        const DATA      = 0x02;
        const RUN       = 0x04;
        const PERM      = 0x80;
        // TODO: More perms for relevant commands
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub what: Allowance,
    pub who: crypto::PublicKey,
}

impl Permission {
    fn allows(&self, what: &Command) -> bool {
        match what {
            Command::Shell(_, _) => self.what.contains(Allowance::RUN),
            Command::ShellOutput(_) => self.what.contains(Allowance::DATA),
            Command::Forward(_) => self.what.contains(Allowance::FORWARD),
            Command::Print(_) => self.what.contains(Allowance::DATA),
        }
    }

    fn allows_grant(&self) -> bool {
        self.what.contains(Allowance::PERM)
    }
}

fn enc_vec<T: serde::Serialize>(
    clears: Vec<(&crypto::PublicKey, T)>,
    sender_skey: &crypto::SecretKey,
) -> Result<Vec<Vec<u8>>> {
    let mut out = Vec::with_capacity(clears.len());
    for (recver_pkey, item) in clears.into_iter() {
        let crypted = crypto::ser_encrypt(&item, sender_skey, recver_pkey)?;
        out.push(crypted);
    }
    Ok(out)
}

// header rows, body rows
type SerdBlob = (Vec<Vec<u8>>, Vec<Vec<u8>>);

pub struct Packet;

impl Packet {
    pub fn serialize(
        sender_skey: &crypto::SecretKey,
        perms: Vec<(&crypto::PublicKey, Permission)>,
        commands: Vec<(&crypto::PublicKey, Command)>,
    ) -> Result<Vec<u8>> {
        crypto::init().map_err(Error::CryptoFail)?;

        println!("Encrypting components");
        let header_rows = enc_vec(perms, sender_skey)?;
        let body_rows = enc_vec(commands, sender_skey)?;
        println!("Serializing tuple ({}, {})", header_rows.len(), body_rows.len());
        bincode::serialize::<SerdBlob>(&(header_rows, body_rows)).map_err(Error::BincodeFail)
    }

    pub fn deserialize(
        from: &[u8],
        sender_pkey: &crypto::PublicKey,
        recver_skey: &crypto::SecretKey
    ) -> Result<Vec<Command>> {
        crypto::init().map_err(Error::CryptoFail)?;

        let (header_rows, footer_rows): SerdBlob = bincode::deserialize(from).map_err(Error::BincodeFail)?;

        let mut perms = vec![Permission {
            what: Allowance::all(),
            who: sender_pkey.clone(),
        }];
        for row in header_rows {
            let decd = perms.iter().find_map(|perm| {
                if !perm.allows_grant() {
                    return None;
                }
                crypto::de_decrypt(&row, &perm.who, recver_skey).ok()
            });
            let decd = match decd {
                Some(decd) => decd,
                None => continue,
            };
            perms.push(decd);
        }

        let mut res = vec![];
        for row in footer_rows {
            let decd = perms.iter().find_map(|perm| {
                let decd = crypto::de_decrypt(&row, &perm.who, recver_skey).ok()?;
                if !perm.allows(&decd) {
                    None
                } else {
                    Some(decd)
                }
            });
            let decd = match decd {
                Some(decd) => decd,
                None => continue,
            };
            res.push(decd);
        }

        Ok(res)
    }
}

#[cfg(feature = "handling")]
pub mod handle;
