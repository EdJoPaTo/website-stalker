#!/usr/bin/env bash

name="website-stalker"

sudo systemctl disable --now "$name.timer" "$name.service"

sudo rm -f "/usr/lib/tmpfiles.d/$name.conf"
sudo rm -f "/usr/lib/sysusers.d/$name.conf"
sudo rm -f "/usr/local/lib/systemd/system/$name.service"
sudo rm -f "/usr/local/lib/systemd/system/$name.timer"
sudo rm -f "/usr/local/bin/$name"

sudo userdel -r "$name"
sudo groupdel "$name"

sudo systemctl daemon-reload


echo "/var/local/lib/website-stalker/ is not touched and is still existing"
