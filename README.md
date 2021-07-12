# Website Stalker

> Track changes on websites via git

This tool checks all the websites listed in its config.
When a change is detected, the new site is added to a git commit.
It can then be inspected via normal git tooling.

Basically its just `curl`, `sed` and then `git commit` in a neat package.

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

The config describes a list of sites.
Each site has an URL and a file extension which is used to save the file.
Additionally, each site can have editors which are used before saving the file.
Each [editor](#editors) manipulates the content of the URL.

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
      - css_selector: article
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

### Editors

#### css_selector

Tries to grab every instance of matching HTML elements and returns all of them (in a still valid HTML).
Optionally with a `remove: true` it returns everything excluding the matching HTML elements.

If no matching HTML elements are found, this editor fails.

Examples:

```yaml
editors:
  - css_selector: article
  - css_selector:
      selector: article
  - css_selector:
      selector: a
      remove: true
  - css_selector: h1 a
  - css_selector: h1 > a
```

#### html_prettify

Formats the input as pretty HTML.

Example:

```yaml
editors:
  - html_prettify
```

#### html_text

Only returns text content of HTML elements within the input.

Example:

```yaml
editors:
  - html_text
```

#### regex_replacer

Searches the input with a RegEx pattern and replaces all occurrences with the given replace phrase.
Grouping and replacing with `$1` also works.

Examples:

```yaml
editors:
  # Remove all occurences of that word
  - regex_replacer:
    pattern: "tree"
    replace: ""
  # Remove all numbers
  - regex_replacer:
    pattern: "\\d+"
    replace: ""
  # Find all css files and remove the extension
  - regex_replacer:
    pattern: "(\\w+)\\.css"
    replace: $1
```

#### rss

Creates an RSS 2.0 Feed from the input.
An RSS item is generated for every `item_selector` result.
The other selectors can be used to find relevant information of the items.
The content is the full result of the `item_selector`.
It can be further edited with with every available [editor](#editors).

Defaults:
- `title`: When a `<title>` exists, it will be used. Otherwise, it's empty.
- `item_selector`: `article`
- `title_selector`: `h2`
- `link_selector`: `a`
- `content_editors` can be omitted when empty

Examples:

```yaml
  # Fully specified
  - url: "https://edjopato.de/post/"
    extension: xml
    editors:
      - rss:
          title: EdJoPaTos Blog
          item_selector: article
          title_selector: h2
          link_selector: a
          content_editors:
            - css_selector:
                selector: "h2, article > a, div"
                remove: true
            - html_text

  # Minimal working example
  - url: "https://edjopato.de/post/"
    extension: xml
    editors:
      - rss: {}
```

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
- [Feed me up, Scotty!](https://gitlab.com/vincenttunru/feed-me-up-scotty) creates RSS feeds from websites
- [urlwatch](https://thp.io/2008/urlwatch/)
