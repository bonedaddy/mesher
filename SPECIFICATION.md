# mesher specification

This document describes the high-level architecture of mesher, as well as the details of the communication protocol they use.
It also describes several types of implants which might be used in a network, as well as advice for optimal configuration.

## Definitions

The words "must", "most not", "required", "shall", "shall not", "should", "should not", "recommended", "may", and "optional" are defined in [RFC2119].
Other words not defined below are used according to their colloquial English meaning.

-   **C2** is short for "command and control".
    It can either mean the people creating and sending commands along the network, or be used as a modifier meaning something used directly by those people.
    For example, "C2 computer" means the specific computer they type on, and _not_ merely computers under their control.
-   An **operation** is a specific purpose for which implants are being controlled.
    C2 defines the operation's purpose and scope.
-   An **adversary** is the person or group of people who oppose the operation.
-   An **implant** is software controlled by C2, running on a non-C2 computer.
    Several types of mesher implants are defined in the _Implant Types_ section.
-   A **target implant** is an implant important to completing the operation.
-   A **compromised** computer is one under the control of the adversary.
-   A **zone** is a conceptual collection of computers.
    They may or may not be technologically interlinked.
    There are several "zones" referenced in this specification.
    These zones are treated as minimal requirements, _not_ as full descriptions.
    For example, a computer in yellow zone may be securely controlled
    -   **Green zone** includes all computers under the direct, secure control of C2.
        They are trusted not to be compromised.
        How this trust is validated is not defined in this specification.
        Green zone computers are assumed to be somehow to C2.
    -   **Yellow zone** are the computers which are controlled by C2.
        Unlike green zone, the control is not certain, but it's considered unlikely for the adversary to have compromised them.
        Yellow zone computers also aren't directly tied to the identity of C2.
    -   **Red zone** are computers under the control of the adversary.
        These are, of course, permanently and irrevocably compromised.
    -   **White zone** computers are not under the direct control of C2.
        For security reasons, this design was created assuming the adversary controls them.
        However, in practice, they're more likely to be controlled by some relatively neutral third party.

## High-level architecture

A mesher network is composed of multiple nodes.
Each node is identical with respect to communication, but may differ in how they handle received messages.

 [RFC2119]: https://www.ietf.org/rfc/rfc2119.txt
