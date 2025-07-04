# loukanikos
Detect defense contractor hardware via Bluetooth and WiFi MAC address filtering.
Named after the Greek protest dog.

The impetus for this project was that we realized lots of cops have WiFi and
Bluetooth enabled Axon body cams, and that we could probably do proximity cop
detection using simple MAC address filtering.
After finding the JSON files of defense contractors
from https://hackthepolice.pages.gay, we realized the scope could be pretty
reasonably expanded to many defense contractors.

MAC addresses are added to the dataset by contributors. We're not done filling
out the dataset yet. If you want to add a new company, or add MAC prefixes to an
existing company, please put in a pull request.

## Milestones
- [X] Basic WiFi capturing using tcpdump
- [ ] Improved capture using `libpcap`
- [ ] Utilization of multiple wireless interfaces
- [ ] Builds for embedded systems (ESP32?)
- [ ] Count for number of unique MACs spotted in the last X minutes
