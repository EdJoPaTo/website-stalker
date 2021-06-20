# Website Stalker systemd units

When the task is to run the website-stalker every once in a while the systemd.timer comes in handy.

## Install

The systemd timer is included within the AUR package since version 0.7.
You can start it via `sudo systemctl enable --now website-stalker.timer`.

If you do not use the AUR clone the repo instead

Check out what the [`install.sh`](install.sh) does.
When you are comfortable with it, you can run it from the main directory:
```bash
./systemd/install.sh
```

## Adapt

You probably want to use a git repo within the working directory of the service.
Head over to `cd /var/local/lib/website-stalker/` (or `/var/lib/website-stalker/` if you installed via AUR) and create your git repo (`git init`).
Also you need to create a config `website-stalker.yaml` in the folder.
Check `website-stalker example-config` or the [main README.md](../README.md) for an example.

You can change the time interval of the systemd timer with the following command:
```bash
sudo systemctl edit website-stalker.timer
sudo systemctl restart website-stalker.timer
```
