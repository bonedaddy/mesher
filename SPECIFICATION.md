# mesher specification

This document describes several things:
First, the system architecture mesher manages;
second, the communication protocol mesher uses, so third-party addons can be created;
third, an explanation of how to use mesher.
It's meant primarily as a reference for people contributing to mesher or implementing compatible products, but should also describe mesher in sufficient detail to analyze the security of the design.
It may also help with using it, by making the underlying concepts easier to understand.

## Definitions

The words "must", "most not", "required", "shall", "shall not", "should", "should not", "recommended", "may", and "optional" are defined in [RFC2119].
Some words are defined in the list below.
Some are defined in the prose.
Other words are used according to their _colloquial_ English meaning.

-   **C2** is short for "command and control".
    It can either mean the people creating and sending commands along the network, or be used as a modifier meaning something used directly by those people.
    For example, "a C2 computer" means the specific computer they type on, and _not_ merely computers under their control.
-   A **mission** is the purpose for which C2 has been established, and their primary goal.
-   An **adversary** is the person or group of people who oppose the mission, and by extension C2.
-   An **implant** is software following the mesher standard.
    It's controlled by C2, but the computer it's on is not, except indirectly through the implant.
    Several types of mesher implants are outlined in the _Implant Types_ section.
-   The **controller** is the software that C2 is using to issue commands to implants.
-   A **node** is a computer on which an implant is running.
    The distinction between node and implant is made because the implant's control of the node can be disrupted.
    Nodes also have a "color". Specifically:
    -   **Green** nodes are those under the direct, secure control of C2.
        They are assumed to be somehow tied to the identity of C2, and therefore not anonymous.
    -   **Blue** nodes are those which C2 securely controls and aren't tied to their identity.
    -   **Yellow** nodes are controlled by C2, but not identifiably so.
        C2's control is weaker than that of green or blue nodes, but still fairly certain.
        It's _possible_ that the adversary has control of these, but unlikely.
    -   **White** nodes are not under the direct control of C2, but which C2 can influence.
        For example, they may belong to a third party like cloud hosting, but be running an implant.
    -   **Red** nodes are computers that belong to the adversary.
        These are, of course, permanently and irrevocably compromised.
    -   **Black** nodes are securely and certainly controlled by the adversary.
-   An **operation** is a specific, individual purpose for which implants are being controlled.
    C2 defines the operation's purpose and scope.
    The meaning analogous to the one used when defining addition as an "operation".
-   A **mesh** is all the nodes used in a given operation.

 [RFC2119]: https://www.ietf.org/rfc/rfc2119.txt

## High-level architecture

mesher, like most anonymization methods, accomplishes its goal by routing through intermediaries.
When given a network toplogy -- a set of zones

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
Using less memory on the reader's side allows implants to consume fewer resources and hide more effectively.

### Header

### Commands

## Examples

