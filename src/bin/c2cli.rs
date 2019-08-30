use sodiumoxide::crypto::box_ as nacl;

use std::io::Write as _;

fn main() {
    let (_, c2_skey) = nacl::keypair_from_seed(&nacl::Seed([0; 32]));
    let (rat1_pkey, _) = nacl::keypair_from_seed(&nacl::Seed([1; 32]));
    let (rat2_pkey, _) = nacl::keypair_from_seed(&nacl::Seed([2; 32]));

    // let packet = mesher::pack_commands(
    //     &c2_skey,
    //     &[
    //         (mesher::Command::Forward("R2".to_string()), &rat1_pkey),
    //         (mesher::Command::Print("R1 print".to_string()), &rat1_pkey),
    //         (
    //             mesher::Command::Shell(
    //                 "echo hi".to_string(),
    //                 rat2_pkey.clone(),
    //             ),
    //             &rat1_pkey,
    //         ),
    //         (mesher::Command::Print("R2 print".to_string()), &rat2_pkey),
    //     ],
    // )
    // .expect("package");

    // std::io::stderr()
    //     .write_all(&packet)
    //     .expect("Failed to write to stdout");
}
