# Website Stalker

> Track changes on websites via git

This tool checks all the websites listed in its config.
When a change is detected, the new site is added to a git commit.
It can then be inspected via normal git tooling.

Basically its just `curl` and then `git commit` in a neat package.

See it [in action](https://github.com/EdJoPaTo/website-stalker-example) (literally in GitHub **Action**s).

## Install

- [GitHub Releases](https://github.com/EdJoPaTo/website-stalker/releases)
- [Arch Linux User Repository (AUR)](https://aur.archlinux.org/packages/website-stalker/)
- [Docker Hub Image ![Docker Hub Image](https://img.shields.io/docker/image-size/edjopato/website-stalker)](https://hub.docker.com/r/edjopato/website-stalker)
- Via rust and cargo: Clone → `cargo install --path .`

## Usage

### GitHub Actions

Check out [website-stalker-example](https://github.com/EdJoPaTo/website-stalker-example) which runs within GitHub actions.

### Locally

- First create a new folder / repo for tracking website changes
    ```bash
    mkdir personal-stalker
    cd personal-stalker
    git init
    ```

- Create the config file which contains all the websites to be stalked.
    Add your favorite website.
    Also make sure to set the value of [from](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From) to an email address of yours.

    ```bash
    website-stalker example-config > website-stalker.yaml
    nano website-stalker.yaml
    ```

- Check if your config is valid
    ```bash
    website-stalker check
    ```

- Run your newly added website. If you added `https://apple.com/newsroom` use something like this to test if everything works like you want:
    ```bash
    website-stalker run apple
    ```

- Set up a cronjob / systemd.timer executing the following command every now and then
    ```bash
    website-stalker run --all --commit
    ```

### Config Example

```yaml
# This is an example config
# The filename should be `website-stalker.yaml`
# and it should be in the working directory where you run website-stalker.
#
# For example run `website-stalker example-config > website-stalker.yaml`.
# And then do a run via `website-stalker run`.
---
from: my-email-address
sites:
  - url: "https://edjopato.de/post/"
    extension: html
    editors:
      - css_selector:
          selector: article
      - css_selector:
          selector: a
          remove: true
      - html_prettify
      - regex_replacer:
          pattern: "(Lesezeit): \\d+ \\w+"
          replace: $1
  - url: "https://edjopato.de/robots.txt"
    extension: txt
```

There is a bigger [config](https://github.com/EdJoPaTo/website-stalker-example/blob/main/website-stalker.yaml) in my [example repo](https://github.com/EdJoPaTo/website-stalker-example).
The example repo is also used by me to detect changes of interesting sites.

### Command Line Arguments

```plaintext
Website Stalker 0.8.0
EdJoPaTo <website-stalker-rust@edjopato.de>
Track changes on websites via git

USAGE:
    website-stalker <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    check             check if the config is fine but do not run
    example-config    Prints an example config which can be piped into website-stalker.yaml
    help              Prints this message or the help of the given subcommand(s)
    run               stalk all the websites you specified
```

```plaintext
website-stalker-check
check if the config is fine but do not run

USAGE:
    website-stalker check
```

```plaintext
website-stalker-example-config
Prints an example config which can be piped into website-stalker.yaml

USAGE:
    website-stalker example-config
```

```plaintext
website-stalker-run
stalk all the websites you specified

USAGE:
    website-stalker run [FLAGS] <site filter>

FLAGS:
        --all        run for all sites
        --commit     git commit changed files
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <site filter>    filter the rules to be run (regular expression)
```

# Alternatives

- [Website Changed Bot](https://github.com/EdJoPaTo/website-changed-bot) is a Telegram Bot which might potentially use this tool later on
- [ChangeDetection](https://github.com/bernaferrari/ChangeDetection) is an Android app for this
- [urlwatch](https://thp.io/2008/urlwatch/)
