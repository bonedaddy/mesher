use sodiumoxide::crypto::box_ as nacl;

use std::io::Write as _;

fn main() {
    let (_, c2_skey) = nacl::keypair_from_seed(&nacl::Seed([0; 32]));
    let (rat1_pkey, _) = nacl::keypair_from_seed(&nacl::Seed([1; 32]));
    let (rat2_pkey, _) = nacl::keypair_from_seed(&nacl::Seed([2; 32]));

    let packet = mesher::Packet::serialize(
        &c2_skey,
        vec![
            (&rat2_pkey, mesher::Permission { what: mesher::Allowance::DATA, who: rat1_pkey.clone() }),
        ],
        vec![
            (&rat1_pkey, mesher::Command::Forward("R2".to_string())),
            (&rat1_pkey, mesher::Command::Print("R1 print".to_string())),
            (
                &rat1_pkey,
                mesher::Command::Shell(
                    "echo hi".to_string(),
                    rat2_pkey.clone(),
                ),
            ),
            (&rat2_pkey, mesher::Command::Print("R2 print".to_string())),
        ]
    ).expect("package");
    println!("packet len: {}", packet.len());

    std::io::stderr()
        .write_all(&packet)
        .expect("Failed to write to stderr");
}
