#!/usr/bin/env bash
set -e

name="website-stalker"

dir=$(basename "$(pwd)")
if [ "$dir" == "systemd" ] || [ "$dir" == "system" ]; then
    echo "run from main directiory like this: ./systemd/system/install.sh"
    exit 1
fi

nice cargo build --release --locked

# systemd
sudo mkdir -p /usr/local/lib/systemd/system/
sudo cp -uv "systemd/system/systemd.service" "/usr/local/lib/systemd/system/$name.service"
sudo cp -uv "systemd/system/systemd.timer" "/usr/local/lib/systemd/system/$name.timer"
sudo cp -uv "systemd/system/sysusers.conf" "/usr/lib/sysusers.d/$name.conf"
sudo cp -uv "systemd/system/tmpfiles.conf" "/usr/lib/tmpfiles.d/$name.conf"
sudo systemd-sysusers
sudo systemd-tmpfiles --create
sudo systemctl daemon-reload

# stop, replace and start new version
sudo systemctl stop "$name.service" "$name.timer"
sudo cp -v "target/release/$name" /usr/local/bin

sudo systemctl enable --now "$name.timer"

echo "you might want to create a git repo (via git init) and a config (via website-stalker example-config > website-stalker.yaml) in the working directory:"
echo "/var/local/lib/website-stalker/"
