# mesher specification

This document describes several things:
First, the system architecture mesher manages;
second, the communication protocol mesher uses, so third-party addons can be created;
third, an explanation of how to use mesher.
It's meant primarily as a reference for people contributing to mesher or implementing compatible products, but should also describe mesher in sufficient detail to analyze the security of the design.
It may also help with using it, by making the underlying concepts easier to understand.

## Definitions

The words "must", "most not", "required", "shall", "shall not", "should", "should not", "recommended", "may", and "optional" are defined in [RFC2119].
Other words not defined below are used according to their _colloquial_ English meaning.

-   **C2** is short for "command and control".
    It can either mean the people creating and sending commands along the network, or be used as a modifier meaning something used directly by those people.
    For example, "C2 computer" means the specific computer they type on, and _not_ merely computers under their control.
-   An **adversary** is the person or group of people who oppose the operation.
-   An **implant** is software following the mesher standard.
    It's controlled by C2, but the computer it's on is not, except indirectly through the implant.
    Several types of mesher implants are defined in the _Implant Types_ section.
-   The **controller** is the software that C2 is using to issue commands to implants.
-   A **node** is a computer on which an implant is running.
    The distinction between node and implant is made because the implant's control of the node can be disrupted.
-   An **operation** is a specific, individual purpose for which implants are being controlled.
    C2 defines the operation's purpose and scope.
    The meaning analogous to the one used when defining addition as an "operation".
-   A **mesh** is all the nodes used in a given operation.
-   A **zone** is a conceptual grouping of nodes which may be technologically connected.
    This idea is orthogonal to that of a mesh -- both are collections of nodes, but meshes are linked by common usage, while zones are linked by common properties.
    There are several "zones" referenced in this specification.
    -   **Green zone** includes all computers under the direct, secure control of C2.
        They are trusted not to be compromised.
        How this trust is validated is not defined in this specification.
        Green zone computers are assumed to be somehow to the identity of C2.
        C2's computer is, by definition, in the green zone.
    -   **Yellow zone** are the computers which are controlled by C2.
        Unlike green zone, the control is not certain, but it's considered unlikely for the adversary to have compromised them.
        Yellow zone computers also aren't tied to the identity of C2.
    -   **Red zone** are computers under the control of the adversary.
        These are, of course, permanently and irrevocably compromised.
    -   **White zone** computers are not under the direct control of C2.
        For security reasons, this design was created assuming the adversary controls them.
        However, in practice, they're more likely to be controlled by some relatively neutral third party.

## High-level architecture

mesher's purpose is to make it easy to securely and anonymously send commands to implants, even through untrusted zones.
It does that by routing encrypted commands through other implants, similar to onion routing, though with a different forwarding scheme that leaks less information.
The tradeoff is that, unlike onion routing, a persistent 'tunnel' can't easily be built.
mesher provides some tooling to automate the process, but 
To some extent, this is intentional, because it makes anonymity easier to uphold if there's no single, consistent path to follow.

TODO: single-way, circuits, etc.

## Communication protocol

The communication protocol is defined around packets.
Each packet can be sent through virtually any transport layer, from plain TCP to steganographically encoded images, as long as it has enough bandwidth.
The packets themselves are designed to expose as little information as possible both to any passive eavesdroppers, as well as any compromised nodes.

Packets can be broadly defined as a group of commands, with meta-information in a header.
This specification defines no maximum packet size, because of a few of the predefined payloads.
However, it is also explicitly spec-compliant to set a limit for a given piece of software.
There _is no_ way to negotiate this limit, as that would require a connection between the source and implant.
mesher provides a setting to error out on packets greater than a given size, but the user is expected to know the maximum packet size their nodes support.

The format is designed to be easily read from a stream, with minimal memory overhead, occasionally at the expense of performance, as well as taking up minimal bandwidth.
Using less memory on the reader's side allows implants to consume fewer resources and 

### Header

### Commands

 [RFC2119]: https://www.ietf.org/rfc/rfc2119.txt
