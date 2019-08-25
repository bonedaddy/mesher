# mesher

mesher is a command and control system, designed to deliver commands securely and anonymously.
Configured properly, the identity of the C2 is multiple steps removed from being discoverable by the implanted host through any means.
This anonymization happens similarly to Tor's anonymization, by redirecting commands through multiple nodes.
It also repeatedly reencrypts portions of the message, ensuring that C2s also can't be tracked by which keys are used.
Finally, all communications are secured through public, well-known crypto:
[libsodium], specifically its [crypto_box functions][crypto_box], which use X25519 to perform a key exchange, then encrypt the message with XSalsa20 and authenticate with Poly1305.
A detailed specification can be found in [the specification][spec]

This project is open-source, published under the [BSD 3-clause][license] license, also available in this repo in LICENSE.md.
The license can be summarized as unrestricted reuse, with credit given, without endorsement, but the only legally binding text is that in LICENSE.md in this repository.
There is also absolutely no warranty of any kind, express or implied.

## Architecture

This crate can be built into one library and four binaries.
The library provides several functions to convert commands into a packet of bytes to send, as well as parsing commands out of a packet. 
It also provides functions for sensible default parsing of received commands, though most implants will need custom command parsing code.

## Usage

Each of the tools has its own usage guide, given how different they are:

TODO

 [libsodium]: https://github.com/jedisct1/libsodium
 [crypto_box]: https://download.libsodium.org/doc/public-key_cryptography/authenticated_encryption
 [license]: https://opensource.org/licenses/BSD-3-Clause
 [spec]: SPECIFICATION.md
