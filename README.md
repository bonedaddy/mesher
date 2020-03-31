# mesher

> **Note**:
> This README is somewhat aspirational.
> It describes, in the present tense, what the mesher project will eventually include.
> Many features are already there -- the library is feature-complete, pending some API revision -- but many more are not.
> For example, some of the crates referenced (`mesher-node` especially) haven't even been started yet, as I wanted a solid API to code against before I tried writing them.
> And, of course, if in the course of writing them I discover my code is clunkier than anticipated, the library, or even the architecture overall, could change.

Mesher is a fairly simple wrapper over a fairly complex concept:
Securely and anonymously sending a message over a heterogeneously connected mesh network.

In smaller, more real words, mesher:

- Operates on mesh networks
- Allows nodes to be connected through a variety of channels (heterogeneous, not homogeneous)
- Prevents intermediate nodes from knowing the source or destination
- Prevents intermediate nodes from knowing the contents of the message

It's primarily designed for anonymous, one-way communication.
However, replies facilitate round-trip communications, which in turn can be used to make a tunnel.

The project is made up of two main pieces and several auxiliary parts.
The two main pieces are the library crate [`mesher`](https://crates.io/crates/mesher), which provides the actual functionality, and [`mesher-node`](https://crates.io/crates/mesher-node), a binary crate wrapping around `mesher` which encapsulates the most common uses.
[`mesher-basic`](https://crates.io/crates/mesher-basic) contains a set of "basic" transports -- things that will be useful for all transports to have, like direct TCP connections.
[`mesher-social`](https://crates.io/crates/mesher-social) has transports which use various online platforms, generally social (e.g. IRC, Discord, Pastebin) to send and receive data.
`mesher-node` has three examples to demonstrate how other applications can interact with it.
<!--
MesherNet is a network of premade mesher nodes, along with a directory of them and their supported transports, for use in Tor-like applications.
-->

## TODO

- Finish **§&nbsp;Overview**
- Rewrite **§&nbsp;Usage** to be about using mesher, instead of how it works

## Versioning

Mesher follows [semver](https://semver.org/), with three caveats:

- If it's not documented, it's exempt from semver.
  This includes items which are listed in the generated documentation, but don't have any actual docs associated with them.
  If you'd like a specific quirk to be documented (and therefore stabilized), feel free to open a feature request.
- For now, while the packet format is still being decided, updates to it are patch-level updates, even if they break compatibility.
- Sufficiently major security fixes, _even ones which break the semver contract_, will always be patch-level bumps.
  This ensures wider distribution than if they were treated the same as any other change.
  Every effort will be made to ensure that the resulting compile errors make it explicitly clear what's happened.

## Usage

Mesher can be used in one of two ways: As a library, or through the `mesher-node` binary.
Most will likely use it through `mesher-node`, but those wanting to create a custom node or embed mesher in another program will use it through the `mesher` library.
Detailed documentation on using each is available through their respective Rust crates.
This section covers general concepts, applicable to both.

A mesher network is made up of, of course, meshers.
Meshers manage transports, both inbound and outgoing, and process packets as they're received.
However, they mostly just coordinate the other pieces:

- Transports (and transport types) do the actual sending and receiving of data
- Packets contain the instructions (and messages) for how to use those transports

### Transports

A transport is a single, monodirectional communication channel: either sending or receiving.
The channel's details are all embedded within a URI, where the scheme indicates the transport type, and the rest of the URL is interpreted depending on the type.
Note that while the prefixes are *standardized* (because most transport types have obvious names), they're specified by the mesher, and it can choose any prefix.

### Packets

Each packet consists of a main body, and several reply paths.
Reply paths are described in **§&nbsp;Reply paths**.
The main body is made up of instructions, which can be either transports or messages.
Each instruction is encrypted with a given node's public key, which is the only way that the target is marked.
Nodes, when they receive packets, simply try to decrypt each instruction in the main body; the ones that succeed are the ones it should pay attention to.

When it sees a transport, the mesher will parse the URI as described in **§&nbsp;Transports**, and send the packet (the exact bytes received, not just the successfully decrypted parts) out along that channel.
When it sees a message, the mesher simply passes it along to the caller, for them to handle as desired.

### Replies

> **Note**:
> The way replies work is subject to change.
> The fundamental idea (sending reply paths ahead, and just appending the reply messages) will stay the same, but the format may change to be more flexible.

Replies are based on the reply paths mentioned briefly in **§&nbsp;Packets**.
Each one is formatted identically to the main body, but treated differently.
The key difference is that they're not decrypted and interpreted on receipt of the packet, just parsed into a list of (encrypted) instructions.

## Crypto

There's no actual crypto code in any mesher project, unless you count simple steganography.
[`rand`](https://crates.io/crates/rand) is used for cryptographically secure randomness.
[`sodiumoxide`](https://crates.io/crates/sodiumoxide) is used to perform the encryption, decryption, signing, and verification.

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
  If a node is compromised, mesher makes no attempt to stop it from seeing *that node's* secret information.
  However, the design means they will not be able to read *other nodes'* secrets.
- Reliability is not a guarantee.
  Mesher should be considered akin to IP: Messages are sent out, and whether they're received depends on the integrity of the intervening network.

If you think mesher should provide any additional guarantees, please raise a feature request.
If it already happens to provide another (useful) guarantee, similarly, raise a feature request to ask if it should be intentionally maintained.

### Disclosure

If you find *any* evidence that *any* of the guarantees are false, please email directly at [disclosure@cybers.eco](mailto:disclosure@cybers.eco), to coordinate a fix and disclosure.
You don't need to prove something is wrong -- just point to where you think the problem might be.
However, the more information you provide, the quicker I can get back to you and fix the problem.

If you find any issues with any crates mesher uses, please responsibly disclose the issue to the creators of the relevant crate.
Please *also* let me know (see **§&nbsp;Disclosure** below), so I can switch to safer code.
