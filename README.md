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

### Disclosure

If you find *any evidence* that *any* of the guarantees are false, please email directly at [disclosure@cybers.eco](mailto:nic@cybers.eco), to coordinate a fix and disclosure.
Please allow up to two weeks for an initial response.

If you find any issues with any crates mesher uses, please responsibly disclose the issue to the creators of the relevant crate.
Please *also* let me know (see **§ Disclosure** below), so I can switch to a safer encryption method.

### Primitives used

The crypto primitives, and their respective implementations, are:

- [Ed25519](https://crates.io/crates/ed25519-dalek) / [X25519](https://crates.io/crates/x25519-dalek) for key generation, key agreement, signing, and verification.
  This uses ed25519-dalek's `verify_strict` functionality. See [A Note on Signature Malleability](https://github.com/dalek-cryptography/ed25519-dalek#a-note-on-signature-malleability) for more information.
- [ChaCha20-Poly1305@openssh.com](http://cvsweb.openbsd.org/cgi-bin/cvsweb/src/usr.bin/ssh/PROTOCOL.chacha20poly1305?annotate=HEAD) through [ring](https://github.com/briansmith/ring) for the actual encryption.







