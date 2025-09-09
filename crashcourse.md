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
consisting of six hexadecimal bytes.
