# mesher

> **Note**: This README is written about what mesher *will* become.
> It's not yet there.
> Take this README as an end goal, not as a current, up-to-date reference.
> 
> It's also not so much documentation as it is a scratch pad for my ideas, but nonetheless feel free to leave issues requesting clarification or suggesting improvements.
> I'll need to write real docs at some point, and that feedback will be useful.
> 
> If this note is still present after 1.0, please open an issue for it to be removed.

Mesher is a system -- primarily a library, but also some executables which use it -- to make anonymized communication between members of a mesh network simpler.
It focuses mostly on one-way message delivery, but does provide a way to give a reply route, so a tunnel could be built, given sufficient forethought.
The project is largely divided into two parts: The primary focus, the mesher library, and the mesher-node binary, which provides a "generally good enough" wrapper around the binary so mesher can be incorporated quickly.

Designed to be secure in transit, mesher messages are encrypted with open-source, well-vetted algorithms and code.
Mesher itself contains no crypto math, instead preferring to use open-source and vetted libraries.
The portions using crypto primitives are well-separated into their own submodule for easy auditing.
Because mesher uses [ring], it encrypts instructions with [chacha20-poly1305@openssh.com].
Keys are derived with [X25519], using keys generated randomly each time.
Randomness is pulled from the OS by [`ring::rand::SystemRandom`].

Designed to be flexible, mesher has a robust plugin system.
The library itself has a rich set of hooks to allow for custom send and receive transports, as well as custom message types.
The `mesher-node` binary takes plugins in shared library form (.so/.dll/.dylib, depending on platform) and so is compatible with any language that can compile to those.

 [ring]: https://github.com/briansmith/ring
 [chacha20-poly1305@openssh.com]: http://cvsweb.openbsd.org/cgi-bin/cvsweb/src/usr.bin/ssh/PROTOCOL.chacha20poly1305?annotate=HEAD
 [X25519]: https://briansmith.org/rustdoc/ring/agreement/index.html
 [`ring::rand::SystemRandom`]: https://briansmith.org/rustdoc/ring/rand/struct.SystemRandom.html

## TODO

Some sections left to do, generally because I can't yet or it wouldn't make sense to think about them yet:

- Versioning system
- Project file structure
- Code structure (e.g. traits)
- API docs

## Structure

The core of this project is the mesher library, which provides:

- Building routes and sending messages along them.
- Unified message reception over every supported transport, with one or more keys.
- Duplicate message suppression.
- Transparently forwarding received messages to next recipients.
- Properly parsing replies, and making them easy to send.
- Managing plugins to 
- Optionally requiring messages to be signed before opening them.
- All of that functionality through a simple Rust library.
- An optional C API, enabled through the `c_api` feature.

There are also three binaries that can be compiled directly from this repo.
Enable them with the `binaries` feature.
Note that building them might take much longer than just the library, so they're off by default.

The first is `mesher-node`, which is installed when you `cargo install mesher`.
It provides a simple but entirely functional node for a mesher network, and essentially just wraps the library in a ready-to-use executable.
It can be communicated with over HTTP to localhost, the details of which are in the **mesher-node** section below.

The second and third are `send` and `recv`, examples which just show how to communicate with `mesher-node`, allowing you to send strings and show received strings.
Because they're designed as demos, they're included as Cargo examples, rather than binaries.
They're likely to be almost useless in "real life", but should show quite clearly how to communicate with `mesher-node`, and even provide some code to pull from.

## Messages

The core of mesher's networking is, of course, the messages it sends.
They're structured as a list of "instructions", which can be one of two types:

- Forward, which indicates where the message should be sent next.
- Data, which contains the data of the message.
- Reply, which contains the key to encrypt the reply data with and indicates that the remainder of the message should be used as the reply-route.

> **TODO**: Settle on and describe the byte format. KISS.

In transit, each instruction is encrypted with its intended recipient's public key.
Any intermediate node can see all of the encrypted instructions, but unless they have a backdoor for Curve25519 or they compromise the key somehow, they won't be able to access the actual contents.
On receipt of a message, the library attempts to decrypt all of the instructions.
Any instructions not encrypted with one of that node's keys will, of course fail, and are ignored, but not discarded.

Messages can also be signed.
Nodes are configured to expect signatures or not, and there's nothing in the message format itself to say whether or not it's signed.
Nodes configured to accept signed messages will only respond at all to messages signed with the correct key, and ignore both unsigned and incorrectly signed messages.
The signature, if present, is the last 64 bytes, signing the raw message rather than any construction of its instructions.

## Routes

The way routes are described is slightly different from things like Tor.
Rather than providing a list of nodes, and a standard way to transport data between nodes, mesher routes are described by lists of transports.
If you're familiar with graph theory, this is analogous to describing a path by its edges.
This opens up two interesting opportunities:

- More transport methods.
  Of coure, when a path is defined only by its nodes, there has to be some standardized way to communicate between them.
  However, by using transport methods to define the route, we can of course use multiple methods along the path.
- Next- and previous-node anonymization.
  While this only applies to certain transports -- e.g. TCP doesn't benefit from this -- by simply defining how messages are sent, we can now use transports which don't expose the identity of adjacent nodes.
  For example, text uploaded to [Pastebin] will not reveal its source or destination to anyone without access to Pastebin's logs.
  If routes were defined by the nodes, then each node would by definition know the next one, since it has to know where to send the message.
- Dead drop-style transports.
  In these cases, the recipient isn't sent a message, but instead is *left* one.
  This is related to the previous point, but more general, as dead drops may be identifiable.
  However, it does mean nodes can't be as easily connected to each other, even if both are being watched, because they most likely won't be accessing the dead drop simultaneously, and will likely be accessing it in different ways.

There are no compromises to this method that affect mesher's goal.
However, it would likely be unsuitable for general mesh networking, mostly because it's more complication than is necessary in most cases.
Most mesh networks will be built out of identical nodes with identical communication capabilities.
Again: mesher makes no attempt to be a general-purpose mesh networking library, and its design reflects that.

While constructing the routes at the sender, more information is needed: Which node should execute each instruction.
Nodes are identified at the sender by their public key, since that's all the sender really cares about.
When the entire bundle is received, the node will try to decrypt each message, and the ones it can, the ones encrytped with its key, are the ones it interprets.
Each transport has to be matched with a public key, but public keys can be reused.
This allows a single node to send the same message out multiple times, along multiple routes, in case part of the route may be unreliable.
Mesher will automatically ignore duplicate messages received, within configurable limits, to make this simpler to handle.

# mesher-node

> **TODO**: Describe how to communicate with this: JSON over HTTP over `localhost:[port]`.

 [Pastebin]: https://pastebin.com/
