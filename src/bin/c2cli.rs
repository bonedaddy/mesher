use sodiumoxide::crypto::box_ as crypto;

use std::io::Write as _;

fn main() {
    let (_, c2_skey) = crypto::keypair_from_seed(&crypto::Seed([0; 32]));
    let (rat1_pkey, _) = crypto::keypair_from_seed(&crypto::Seed([1; 32]));
    let (rat2_pkey, _) = crypto::keypair_from_seed(&crypto::Seed([2; 32]));

    let packet = mesher::pack_commands(
        &c2_skey,
        &[
            (mesher::Command::Custom(vec![1, 2, 3, 4]), &rat1_pkey),
            (mesher::Command::Custom(b"item 2".to_vec()), &rat2_pkey),
            (mesher::Command::Forward("item 3".to_string()), &rat1_pkey),
        ],
    )
    .expect("package");

    std::io::stdout()
        .write_all(&packet)
        .expect("Failed to write to stdout");
}
