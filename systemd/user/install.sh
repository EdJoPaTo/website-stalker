#!/usr/bin/env bash
set -e

name="website-stalker"

dir=$(basename "$(pwd)")
if [ "$dir" == "systemd" ] || [ "$dir" == "user" ]; then
    echo "run from main directiory like this: ./systemd/user/install.sh"
    exit 1
fi

# Create working directory (inspired by $XDG_DATA_HOME but using it would probably be messy in the .service file -> keep it simple)
mkdir -p "$HOME/.local/share/$name"
# Create config/bin folders
CONFIG_DIR=${XDG_CONFIG_DIRS:-"$HOME/.config"}
mkdir -p "$CONFIG_DIR/systemd/user/" "$HOME/.local/bin"

nice cargo build --release --locked

# systemd
cp -uv "systemd/user/systemd.service" "$CONFIG_DIR/systemd/user/$name.service"
cp -uv "systemd/user/systemd.timer" "$CONFIG_DIR/systemd/user/$name.timer"
systemctl --user daemon-reload

# stop, replace and start new version
systemctl --user stop "$name.service" "$name.timer"
cp -v "target/release/$name" "$HOME/.local/bin"

# You probably only need one
systemctl --user enable --now "$name.service"
systemctl --user enable --now "$name.timer"
