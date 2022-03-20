# Website Stalker systemd units

When the task is to run the website-stalker every once in a while the systemd `timer` comes in handy.

## Install

The systemd timer is included within the AUR package since version 0.7.
You can start it as a system timer via `sudo systemctl enable --now website-stalker.timer`.
The user timer is available since version 0.7.1 and can be started via `systemctl --user enable --now website-stalker.timer`.

If you do not use the AUR clone the repo instead:

Check out what the `install.sh` does in the `system` or `user` directory depending on what you want to install.
When you are comfortable with it, you can run it from the main directory:

```bash
# Choose the one you want
./systemd/system/install.sh
./systemd/user/install.sh
```

## Adapt

You probably want to use a git repo within the working directory of the service.
Head over to the working directory (see a list of possible locations below) and create your git repo (`git init`).
Also, you need to create a config `website-stalker.yaml` in the folder.
Check `website-stalker example-config` or the [main README.md](../README.md) for an example.

Possible working directory locations depending on your installation:

- system service: `/var/lib/website-stalker/`
- user: `$HOME/.website-stalker/`

You can change the time interval of the systemd timer with the following commands (depending on your installation):

```bash
# System timer
sudo systemctl edit website-stalker.timer
sudo systemctl restart website-stalker.timer

# User timer
systemctl --user edit website-stalker.timer
systemctl --user restart website-stalker.timer
```
