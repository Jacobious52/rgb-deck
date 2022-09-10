#!/bin/bash

set -eux

uf2="$1"

diskutil unmount /dev/disk4s1
mkdir /Volumes/rpi-rp2
mount -w -t msdos /dev/disk4s1 /Volumes/rpi-rp2
cp "$uf2" /Volumes/rpi-rp2/
