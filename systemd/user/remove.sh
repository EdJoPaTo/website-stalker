#!/usr/bin/env bash

name="website-stalker"

systemctl --user disable --now "$name.timer" "$name.service"

CONFIG_DIR=${XDG_CONFIG_DIRS:-"$HOME/.config"}
rm -f "$CONFIG_DIR/systemd/user/$name.service"
rm -f "$CONFIG_DIR/systemd/user/$name.timer"
rm -f "$HOME/.local/bin/$name"

systemctl --user daemon-reload


echo "$HOME/.local/share/website-stalker/ is not touched and is still existing"
