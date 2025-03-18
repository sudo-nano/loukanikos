#!/bin/bash

# Include name of interface as first argument.

# bluelog -v -n -m | awk '{print $3}' | grep -i 00:25:DF | while read line;

# Don't forget to add something that takes the wifi interface and puts it in
# monitor mode
sudo ip link set $1 down
sudo iw dev $1 set type monitor
sudo ip link set $1 up

# Return interface to managed mode on SIGINT / Ctrl + C
trap "sudo ip link set $1 down; sudo iw dev $1 set type managed; sudo ip link set $1 up; echo 'Exiting.'" SIGINT

# The organizationally unique identifier (OUI) for Axon/Taser International is 00:25:DF
sudo tcpdump -e -i $1 | grep -i -E '00:25:DF:..:..:..' | while read line;
do
    echo "Axon MAC detected!"
done

# Ideas for later:
# - Send push notification to phone
# - Record interval between cop entering area and leaving area

# TODO
# - Set up trap for SIGINT
# - Filter tcpdump output for MAC addresses (regex?)
