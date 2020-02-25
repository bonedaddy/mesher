# mesher

> **Note**:
> This README is somewhat aspirational.
> It describes, in the present tense, what the mesher project will eventually include.
> Many features are already there -- e.g. the library is nearly feature-complete -- but many more are not.

Mesher is a fairly simple wrapper over a fairly complex concept:
Securely and anonymously sending a message over a heterogeneously connected mesh network.

In smaller, more real words, mesher:

- Operates on mesh networks
- Allows nodes to be connected through a variety of channels (heterogeneous, not homogeneous)
- Prevents intermediate nodes from knowing the source or destination
- Prevents intermediate nodes from knowing the contents of the message

It's primarily designed for anonymous, one-way communication.
However, replies facilitate round-trip communications, which in turn can be used to make a tunnel.

## TODO

- Finish **§ Overview**
- Write **§ Versioning**, **§ Usage**, and **§ Structure**
- Make sure **§ Crypto** makes sense at all

## Usage

Mesher can be used in one of two ways: As a library, or through the `mesher-node` binary.

### Library

// TODO

### Binary

// TODO

## How it works

It works by bouncing packets between nodes.

Each packet is effectively a list of instructions.
Each instruction is meant for one node, whose public key the instruction is encrypted with, but one node can receive more than one instruction.

## Crypto

Mesher does not implement or directly use any crypto protocols.
Instead, if uses `secretbox` from [`sodiumoxide`](https://crates.io/crates/sodiumoxide).
If you'd like to vet the actual crypto, please vet `sodiumoxide`.

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
- Mesher doesn't (and, realistically, can't) secure nodes against the computers they run on.
  If a node is compromised, mesher makes no attempt to stop them from seeing *that node's* secret information.
  However, the design means they will not be able to read *other nodes'* secrets.
- Reliability is not a guarantee.
  Mesher should be considered akin to IP: Messages are sent out, and whether they're received depends on the integrity of the intervening network.

### Disclosure

If you find *any evidence* that *any* of the guarantees are false, please email directly at [disclosure@cybers.eco](mailto:disclosure@cybers.eco), to coordinate a fix and disclosure.
Please include as much information as you can, and allow up to two weeks for an initial response.
A working exploit would be ideal, but isn't necessary.

If you find any issues with any crates mesher uses, please responsibly disclose the issue to the creators of the relevant crate.
Please *also* let me know (see **§ Disclosure** below), so I can switch to a safer encryption method.
