# mesher

mesher is a command and control system, designed to deliver commands securely and anonymously.
Configured properly, the identity of the C2 is multiple steps removed from being discoverable by the implanted host through any means.
This anonymization happens similarly to Tor's anonymization, by redirecting commands through multiple nodes.
It also repeatedly reencrypts portions of the message, ensuring that C2s also can't be tracked through keys.

## Architecture

This crate can be built into one library and four binaries.
The library provides several functions to convert commands into a packet of bytes to send, as well as parsing commands out of a packet. 
It also provides functions for sensible default parsing of received commands, though most implants will need custom command parsing code.

## Usage

Each of the tools has its own usage guide, given how different they are:

- [`c2cli`](c2cli_usage.md)
- [`c2webui`](c2webui_usage.md)
- [`demo-rat`](demo_rat_usage.md)
- [`demo-fwd`](demo_fwd_usage.md)
- [`mesher` library](lib_docs.md)
