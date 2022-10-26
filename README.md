# Website Stalker

> Track changes on websites via git

This tool checks all the websites listed in its config.
When a change is detected, the new site is added to a git commit.
It can then be inspected via normal git tooling.

Basically it's `curl`, [`sed`++](#editors) and then `git commit` in a neat package.

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
    website-stalker init
    ```

    `website-stalker init` will create a git repo (`git init`) and the example config (`website-stalker example-config > website-stalker.yaml`) for you.

- Add your favorite website to the config file `website-stalker.yaml`.
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

- Set up a cronjob / [systemd.timer](systemd) executing the following command every now and then
    ```bash
    website-stalker run --all --commit
    ```

### Config Example

The config describes a list of sites.
Each site has a URL.
Additionally, each site can have editors which are used before saving the file.
Each [editor](#editors) manipulates the content of the URL.

```yaml
# This is an example config
# The filename should be `website-stalker.yaml`
# and it should be in the working directory where you run website-stalker.
#
# For example run `website-stalker example-config > website-stalker.yaml`.
# Adapt the config to your needs and set the FROM email address which is used as a request header:
# https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From
#
# And then do a run via `website-stalker run --all`.
---
from: my-email-address
sites:
  - url: "https://edjopato.de/post/"
    editors:
      - css_select: article
      - css_remove: a
      - html_prettify
      - regex_replace:
          pattern: "(Lesezeit): \\d+ \\w+"
          replace: $1
  - url: "https://edjopato.de/robots.txt"
```

There is a bigger [config](https://github.com/EdJoPaTo/website-stalker-example/blob/main/website-stalker.yaml) in my [example repo](https://github.com/EdJoPaTo/website-stalker-example).
The example repo is also used by me to detect changes of interesting sites.

### Global Config Options

Options which are globally configured at the root level of the configuration file `website-stalker.yaml`.

#### `from`

Used as the [`From` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From) in the web requests.
It is a required field.

The idea here is to provide a way for a website host to contact whoever is doing something to their web server.
As this tool is self-hosted and can be run as often as the user likes this can annoy website hosts.
While this tool is named "stalker" and is made to track websites it is not intended to annoy people.

This tool sets the [`User-Agent` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent) always to `website-stalker/<version> https://github.com/EdJoPaTo/website-stalker` and the [`From` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From) to the config value.
This way both the creator and the user of this tool can be reached in case of problems.

```yaml
from: my-email-address
```

Alternatively you can specify FROM via environment variable

```bash
export WEBSITE_STALKER_FROM=my-email-address
```

#### `notification_template`

When using the [notifications](#notifications) you might want to use your own style of notification instead of the default one.
You can specify your own template which is handled via the [Mustache Syntax](https://mustache.github.io/mustache.5.html).
The following example contains all currently available data points.

When writing your own template use `website-stalker check` to ensure the template will work.

```yaml
notification_template: |
  These {{siteamount}} sites changed:
  {{#sites}}
  - {{.}}
  {{/sites}}

  The following domains are involved:
  {{#domains}}
  - {{.}}
  {{/domains}}

  {{#singledomain}}
  All changes happened on only one domain: {{singledomain}}
  {{/singledomain}}
  {{^singledomain}}
  The changes happened on various domains.
  {{/singledomain}}

  The {{commit}} contains all these changes.
```

### Per Site Config Options

Options available per site besides the [editors](#editors) which are explained below.

#### `url`

One or multiple URLs can be specified.
The simple form is a single URL:

```yaml
sites:
  - url: "https://edjopato.de/"
  - url: "https://edjopato.de/post/"
```

It's also possible to specify multiple URL at the same time.
This is helpful when multiple sites are sharing the same options (like editors).

```yaml
sites:
  - url:
      - "https://edjopato.de/"
      - "https://edjopato.de/post/"
```

#### `accept_invalid_certs`

Allows HTTPS connections with self-signed or invalid / expired certificates.

From [reqwests documentation](https://docs.rs/reqwest/0.11.4/reqwest/struct.ClientBuilder.html#method.danger_accept_invalid_certs):

> You should think very carefully before using this method. If
> invalid certificates are trusted, *any* certificate for *any* site
> will be trusted for use. This includes expired certificates. This
> introduces significant vulnerabilities, and should only be used
> as a last resort.

Do you have a need for self-signed certificates or the usage of the system certificate store?
Please share about it in [Issue #39](https://github.com/EdJoPaTo/website-stalker/issues/39).

```yaml
sites:
  - url: "https://edjopato.de/post/"
    accept_invalid_certs: true
```

#### `ignore_error`

Only show warning when the site errors.

This is useful for buggy services which are sometimes just gone or for pages which will exist in the future but are not there yet.
Personal example: A bad DNS configuration which lets the website appear non existent for some time.

This setting also skips errors from editors.

```yaml
sites:
  - url: "https://edjopato.de/might-appear-in-the-future"
    ignore_error: true
```

#### `headers`

Add additional [HTTP headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers) to the request to the given site.

This is useful for sites that respond differently based on different headers.
Each header Key/Value pair is supplied as YAML String separated with a `: ` followed by a space in the config.

This is the same syntax as HTTP uses which sadly collides with YAML.
YAML assumes something with a `:` is an object.
Therefor you have to make sure to quote the headers.
Using a YAML object / key/value pair is also not possible as some header keys are allowed multiple times.

```yaml
sites:
  - url: "https://edjopato.de/"
    headers:
      - "Cache-Control: no-cache"
      - "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:106.0) Gecko/20100101 Firefox/106.0"
```

### Editors

Editors are manipulating the content of a webpage to simplify comparing them later on.

For example: If you are interested in the content of a webpage the `<head>` with changing stylesheets isn't interesting to you.
When keeping it, it will still create diffs which end up being commits.
This will create noise you're probably just going to ignore.
That's why editors exist.

Think of editors like a pipeline, the next one gets the input of the one before.
As some editors are assuming HTML input, they won't work (well) with non HTML input.
For example its kinda useless to use `html_prettify` after `html_textify` as text won't end up being pretty HTML.
For this reason editors like `css_select` are still producing valid HTML output.

There are probably more tasks out there that might be useful as editors.
Feel free to provide an issue for an editor idea or create a Pull Request with a new editor.

#### `css_remove`

Tries to remove every instance of matching HTML elements and returns the remaining HTML.
Opposite of [`css_select`](#css_select).

Examples:

```yaml
editors:
  - css_remove: article
  - css_remove: h1 a
  - css_remove: h1 > a
```

#### `css_select`

Use [CSS Selectors](https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/Selectors) to grab every instance of matching HTML elements and returns all of them.

If no matching HTML elements are found, this editor errors.

Examples:

```yaml
editors:
  - css_select: article
  - css_select: h1 a
  - css_select: h1 > a
```

#### `html_markdownify`

Formats the input HTML as Markdown.

This is rather simple right now.
Please report issues you find.

Example:

```yaml
editors:
  - html_markdownify
```

#### `html_prettify`

Formats the input HTML as pretty HTML.

Example:

```yaml
editors:
  - html_prettify
```

#### `html_sanitize`

Strip down HTML to its minimal form.

Example:

```yaml
editors:
  - html_sanitize
```

#### `html_textify`

Only returns text content of HTML elements within the input.

Example:

```yaml
editors:
  - html_textify
```

#### `html_url_canonicalize`

Parses the input HTML for URLs.
URLs are parsed into their canonical, absolute form.

Example:

```yaml
editors:
  - html_url_canonicalize
```

#### `json_prettify`

Formats the input JSON as pretty JSON.

Example:

```yaml
editors:
  - json_prettify
```

#### `regex_replace`

Searches the input with a Regex pattern and replaces all occurrences with the given replace phrase.
Grouping and replacing with `$1` also works.

Examples:

```yaml
editors:
  # Remove all occurences of that word
  - regex_replace:
    pattern: "tree"
    replace: ""
  # Remove all numbers
  - regex_replace:
    pattern: "\\d+"
    replace: ""
  # Find all css files and remove the extension
  - regex_replace:
    pattern: "(\\w+)\\.css"
    replace: $1
```

#### `rss`

Creates an RSS 2.0 Feed from the input.
An RSS item is generated for every `item_selector` result.
The other selectors can be used to find relevant information of the items.
The content is the full result of the `item_selector`.
It can be further edited with every available [editor](#editors).

Defaults:
- `title`: When a `<title>` exists, it will be used. Otherwise, it's empty.
- `item_selector`: `article`
- `title_selector`: `h2`
- `link_selector`: `a`
- `content_editors` can be omitted when empty

Examples:

```yaml
  # Fully specified example
  - url: "https://edjopato.de/post/"
    editors:
      - rss:
          title: EdJoPaTos Blog
          item_selector: article
          title_selector: h2
          link_selector: a
          content_editors:
            - css_select:
                selector: "h2, article > a, div"
                remove: true
            - html_textify

  # Minimal working example
  - url: "https://edjopato.de/post/"
    editors:
      - rss: {}
```

### Notifications

When changes on websites are detected they get saved to filesystem.
When `--commit` is given a git commit is created.

Additionally you can get notified via Telegram, Slack, E-Mail, ...
[pling](https://github.com/EdJoPaTo/pling) is used to send these notifications.
Check its documentation about which environment variables to specify in order to get notifications.

Example with Telegram:
```bash
export TELEGRAM_BOT_TOKEN='123:ABC'
export TELEGRAM_TARGET_CHAT='1234'
website-stalker run --all
```

### Command Line Arguments

```plaintext
Website Stalker 0.16.0

EdJoPaTo <website-stalker-rust@edjopato.de>

Track changes on websites via git

USAGE:
    website-stalker <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    check             Check if the config is fine but do not run
    example-config    Print an example config which can be piped into website-stalker.yaml
    help              Print this message or the help of the given subcommand(s)
    init              Initialize the current directory with a git repo and a config (website-stalker.yaml)
    run               Stalk all the websites you specified
```

```plaintext
website-stalker-check

Check if the config is fine but do not run

USAGE:
    website-stalker check [OPTIONS]

OPTIONS:
    -h, --help            Print help information
        --print-yaml      Print out valid config as yaml
        --rewrite-yaml    Write valid config as website-stalker.yaml
```

```plaintext
website-stalker-example-config

Print an example config which can be piped into website-stalker.yaml

USAGE:
    website-stalker example-config
```

```plaintext
website-stalker-init

Initialize the current directory with a git repo and a config (website-stalker.yaml)

USAGE:
    website-stalker init
```

```plaintext
website-stalker-run

Stalk all the websites you specified

USAGE:
    website-stalker run [OPTIONS] [site filter]

ARGS:
    <site filter>    Filter the sites to be run (case insensitive regular expression)

OPTIONS:
        --all       run for all sites
        --commit    git commit changed files
    -h, --help      Print help information
```

# Alternatives

- [Website Changed Bot](https://github.com/EdJoPaTo/website-changed-bot) is a Telegram Bot which might potentially use this tool later on
- [bernaferrari/ChangeDetection](https://github.com/bernaferrari/ChangeDetection) is an Android app for this
- [dgtlmoon/changedetection.io](https://github.com/dgtlmoon/changedetection.io) can be selfhosted and configured via web interface
- [Feed me up, Scotty!](https://gitlab.com/vincenttunru/feed-me-up-scotty) creates RSS feeds from websites
- [htmlq](https://github.com/mgdm/htmlq) command line tool to format / select html (like jq for html)
- [urlwatch](https://thp.io/2008/urlwatch/)
