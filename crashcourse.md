# Contributor Crash Course
So you want to contribute to loukanikos? Great!
Network programming, and especially wireless, can be spooky for those that are
not familiar with it. Hell, they can be spooky for people who *are* experienced!
This document will walk you through the basics of how this program works, and
you'll be contributing in no time.

## WiFi and MAC Addresses
When your devices communicate using WiFi, their communications are broken up into
frames before they're sent over the air.
Each frame contains a MAC header, payload, and frame check sequence.
MAC is short for Medium Access Control. (Medium as in physical medium, not as in
small medium large.)
The MAC header contains, among other things, the source (sender) address and the
destination (recipient) address. Each wireless interface (the thing on your device
actually sending the radio signals) has a unique MAC address that identifies it,
consisting of six hexadecimal bytes.[^1] People typically write them as colon separated
or dash separated.

> aa:bb:cc:dd:ee:ff

> aa-bb-cc-dd-ee-ff

The beginning of an interface's MAC address is its manufacturer's Organizationally
Unique Identifier (OUI), which we'll call a prefix. Manufacturers can have more than
one of these. In short, the interface is constantly announcing who made it, and
there are public databases that match MAC prefixes to company names.

## The pcap library

[^1]: The reason we specify interface here is that a single device can have multiple interfaces. For example, most devices these days have one WiFi interface and one bluetooth interface.
