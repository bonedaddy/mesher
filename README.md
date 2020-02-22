# mesher

> **Note**:
> This README is somewhat aspirational.
> It describes, in the present tense, what the mesher project will eventually include.
> Many features are already there -- e.g. the library is nearly feature-complete -- but many more are not.
> 
> This also applies to the docs, especially those regarding the crypto subsystem, which is currently *intentionally* utterly broken.

Mesher is a fairly simple wrapper over a fairly complex concept:
Securely and anonymously sending a message over a heterogeneously connected mesh network.

In smaller, more real words, mesher:

- Operates on mesh networks
- Allows nodes to be connected through a variety of channels (heterogeneous, not homogeneous)
- Prevents intermediate nodes from knowing the source or destination
- Prevents intermediate nodes from knowing the contents of the message

It's primarily designed for anonymous, one-way communication.
However, replies facilitate round-trip communications, which in turn can be used to make a tunnel.

## Overview

Mesher sends messages across mesh networks.
It does this by creating packets, then bouncing them between nodes.
The packets are made up of encrypted instructions

## Using mesher

- `#![no_std]`
- 

## Crypto details

If you plan to vet this for production usage which depends on the cryptographic strength, I recommend vetting the code directly.
This will ensure that any inaccurate descriptions here don't affect the vetting.
The code dealing directly with crypto primitives is intentionally separated out into its own file, [`crypto.rs`](https://github.com/nic-hartley/mesher/blob/master/mesher/src/crypto.rs).
This makes it easy to review the use of crypto primitives, though to decide whether any product is truly secure, the entire thing must be vetted, not just its use of primitives.

If you plan to vet this, you should also vet the crates used to do the actual math, and ensure they uphold the security you want.

### Guarantees

Mesher provides several security guarantees:

- Messages cannot be read by anyone except people possessing the relevant secret key.
- Intermediary nodes can't tell where in the path they are.
- Only the source can know the entire path and the destination.
- The destination cannot know the source, though it may be implied by which key signs its instructions.

Note some important exceptions and caveats:

- Some paths, especially if replies are sent along the reverse of the original path, will allow intermediary nodes to infer how far along the path they are through timing analysis.
- Privacy depends on the privacy of the secret keys.
- The guarantees only apply to mesher itself.
  For example, it's entirely possible to leak additional information through the contents of messages sent.
- Mesher does not attempt to (and, realistically, cannot) provide security for the individual nodes against the computer they're running on.
  If, for example, you create and send a message from a compromised computer, it's unrealistic to expect mesher to be able to prevent the adversary from intercepting it, one way or another.
  Similarly, if an adversary controls any given node, mesher makes no attempt to stop them from seeing *that node's* secret information.
  However, the design is such that if the adversary compromises an intermediary node, they will not be able to read *other nodes'* secrets.
- Reliability is not a guarantee.
  Mesher should be considered akin to IP: Messages are sent out, and whether they're received depends on the integrity of the intervening network.
  However, it does implement enough functionality that it would technically be possible to implement TCP over it.
  Have fun, I guess?  

### Disclosure

If you find *any evidence* that *any* of the guarantees are false, please email directly at [disclosure@cybers.eco](mailto:nic@cybers.eco), to coordinate a fix and disclosure.
Please allow up to two weeks for an initial response.

If you find any issues with any crates mesher uses, please responsibly disclose the issue to the creators of the relevant crate.
Please *also* let me know (see **§ Disclosure** below), so I can switch to a safer encryption method.

### Operation description

The crypto primitives, and their respective implementations, are:

- [Ed25519](https://crates.io/crates/ed25519-dalek) for key generation, signing, and verification.
- [X25519](https://crates.io/crates/x25519-dalek) for key agreement over ECDH.
- AES-256-GCM through [ring](https://github.com/briansmith/ring) for encryption.

Mesher operates on a very simple interface, to abstract away the details of the crypto to just one part of the code.
It defines four operations: Encryption, decryption, signing, and verifying.
As expected, encryption and verifying require a public key and a "ciphertext", and decryption and signing require a secret key and a "plaintext".
Even though with signing and verification, no data is technically hidden, I still use the terms "ciphertext" and "plaintext" to distinguish between signed and unsigned data, mostly out of (bad?) habit.

#### Nonces

At program startup, a global 96-bit nonce value is set to the number of nanoseconds since the Unix epoch.
If the OS doesn't support that much precision, then it's approximated by e.g. multiplying milliseconds by a million.
By using a monotonically increasing timescale, as opposed to local time, this prevents accidental nonce reuse if e.g. the user moves to another time zone.

Two meshers, created within the same OS time-step but in different processes, communicating to the same public keys, have a high chance of reusing nonces.
However, because the encryption uses a random key every time -- a random keypair is used by the sender -- the odds of nonce *and key* reuse is extremely low.
The two processes would have to be constructing different messages to the same recipient, at the same point in the message sequence, and happen to randomly generate the same sender keypair.
This is possible with a compromised OS, but mesher explicitly does not include that in its threat model.

Every time a nonce is needed, this global nonce is incremented atomically.
Because mesher isn't expected to be generating nonces more than a billion times a second, on a non-malicious system, this will most likely never reuse a nonce: the starting nonce for a given time should always be "further ahead" than the current global nonce.


#### Encryption and decryption

To encrypt data D to a given target public key T<sub>p</sub>:

1. Generate a random pair of 256-bit keys (R<sub>p</sub>, R<sub>s</sub>).
2. Use ECDG between R<sub>s</sub> and T<sub>p</sub> to get a shared 256-bit secret, S.
3. Encrypt D under AES-256-GCM with key S, with a newly generated nonce (see **§ Nonces**).
   Note that the crypto library being used for this automatically includes the nonce in the resulting ciphertext.
4. Prepend the encrypted message with the 32 bytes of R<sub>p</sub>.

To decrypt a ciphertext C meant for a target secret key T<sub>s</sub>:

1. Split C at the 32-byte mark.
   Take the first 32 bytes as a public key R<sub>p</sub>, and the rest as an encrypted message C<sub>m</sub>.
2. Use ECDF between T<sub>s</sub> and R<sub>p</sub> to get a shared 256-bit secret, S.
3. Decrypt C<sub>m</sub> with key S with AES-256-GCM.
   The crypto library being used for AES-GCM automatically stores the nonce as part of the ciphertext during encryption, so during decryption, mesher doesn't need to explicitly retrieve the nonce.

If any step fails during encryption or decryption (e.g. attempting to decrypt a C fewer than 32 bytes long) then it stops and an error is returned.

#### Signing and verification

Signing and verification are done just by calling `ed25519-dalek`'s relevant methods: `ed25519_dalek::PublicKey::verify_strict` and `ed25519_dalek::ExtendedSecretKey::sign`.
The signature, a fixed-width 64-byte value, is appended to the plaintext to produce the ciphertext.
Verification involves just removing the last 64 bytes of the ciphertext and checking the remainder of the ciphertext against it.

The only major thing to note is that verification is done with `ed25519_dalek::PublicKey::verify_strict`, rather than the normal `verify` method.
See [A Note on Signature Malleability](https://github.com/dalek-cryptography/ed25519-dalek#a-note-on-signature-malleability) for more information.
