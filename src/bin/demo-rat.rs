use sodiumoxide::crypto::box_ as crypto;

use std::io::Read as _;

fn main() {
    let (c2_pkey, _) = crypto::keypair_from_seed(&crypto::Seed([0; 32]));
    let (_, rat_skey) = crypto::keypair_from_seed(&crypto::Seed([1; 32]));

    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).expect("Failed to read");
    mesher::handle::packet(&c2_pkey, &rat_skey, &bytes, |custom| {
        println!("Custom data: {:?}", custom);
    }, |dest, data| {
        println!("Would send {:?} to {}", data, dest);
        true
    }).expect("something went wrong");
}
