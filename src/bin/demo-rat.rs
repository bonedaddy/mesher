use sodiumoxide::crypto::box_ as crypto;

use std::io::Read as _;

fn main() {
    let (c2_pkey, _) = crypto::keypair_from_seed(&crypto::Seed([0; 32]));
    let num = std::env::args().collect::<Vec<_>>()[1].parse().unwrap();
    let (_, rat_skey) = crypto::keypair_from_seed(&crypto::Seed([num; 32]));

    println!("R{} listening", num);

    let mut bytes = Vec::new();
    std::io::stdin()
        .read_to_end(&mut bytes)
        .expect("Failed to read");

    mesher::handle::packet(&c2_pkey, &rat_skey, &bytes, mesher::handle::send)
        .expect("something went wrong");
}
