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
- [Docker Hub Image](https://hub.docker.com/r/edjopato/website-stalker)
- Via rust and cargo: Clone → `cargo install --path .`

## Usage

### GitHub Actions

Check out [website-stalker-example](https://github.com/EdJoPaTo/website-stalker-example) which runs within GitHub actions.

### Locally

- First create a new folder / git repository for tracking website changes

    ```bash
    mkdir personal-stalker
    cd personal-stalker
    git init
    website-stalker example-config > website-stalker.yaml
    ```

- Add your favorite website to the configuration file `website-stalker.yaml`.
    Also make sure to set the value of [from](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From) to an email address of yours.

    ```bash
    website-stalker example-config > website-stalker.yaml
    nano website-stalker.yaml
    ```

- Run your newly added website. If you added `https://apple.com/newsroom` use something like this to test if everything works like you want:

    ```bash
    website-stalker run apple
    ```

- Set up a cronjob / [`systemd.timer`](systemd) executing the following command occasionally

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

There is a bigger [config](https://github.com/EdJoPaTo/website-stalker-example/blob/main/website-stalker.yaml) in my [example repository](https://github.com/EdJoPaTo/website-stalker-example).
The example repository is also used by me to detect changes of interesting sites.

### Global Options

Options which are globally configured at the root level of the configuration file `website-stalker.yaml`.

#### `from`

Used as the [`From` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From) in the web requests.
It is a required field.

The idea here is to provide a way for a website host to contact whoever is doing something to their web server.
As this tool is self-hosted and can be run as often as the user likes this can annoy website hosts.
While this tool is named "stalker" and is made to track websites it is not intended to annoy people.

This tool sets the [`User-Agent` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent) to `website-stalker/<version> https://github.com/EdJoPaTo/website-stalker` and the [`From` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/From) to the user configured value.
This way both the creator and the user of this tool can be reached in case of problems.

```yaml
from: my-email-address
```

Alternatively you can specify FROM via environment variable

```bash
export WEBSITE_STALKER_FROM=my-email-address
```

### Per Site Options

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

From [`reqwests` documentation](https://docs.rs/reqwest/0.11.26/reqwest/struct.ClientBuilder.html#method.danger_accept_invalid_certs):

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

#### `http1_only`

Only use HTTP/1 for the web request.

Back-ends might use HTTP/2 fingerprinting which could result in different or unusable output depending on what the back-end assumes about the client.
HTTP/1 is a simpler protocol which does not allow such kinds of back-end optimizations.

```yaml
sites:
  - url: "https://edjopato.de/post/"
    http1_only: true
```

#### `ignore_error`

Only show warning when the site errors.

This is useful for buggy services which are sometimes just gone or for pages which will exist in the future but are not there yet.
Personal example: A bad DNS configuration which lets the website appear nonexistent for some time.

This setting also skips errors from editors.

```yaml
sites:
  - url: "https://edjopato.de/might-appear-in-the-future"
    ignore_error: true
```

#### `filename`

Overrides the URL based default filename of the site.

Normally the filename is automatically derived from the URL.
For the following example it would be something like `de-edjopato-api-token-0123456789-action-enjoy-20weather.html`.
With the `filename` options it is saved as `de-edjopato-api-weather.html` instead.

```yaml
sites:
  - url: "https://edjopato.de/api?token=0123456789&action=enjoy%20weather"
    filename: de-edjopato-api-weather
```

#### `headers`

Add additional [HTTP headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers) to the request to the given site.

This is useful for sites that respond differently based on different headers.
Each header Key/Value pair is supplied as YAML String separated with a `:` followed by a space.

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

For example: If you are interested in the content of a webpage the `<head>` with changing style-sheets isn't interesting to you.
When keeping it, it will still create diffs which end up being commits.
This will create noise you're probably just going to ignore.
That's why editors exist.

Think of editors like a pipeline, the next one gets the input of the one before.
As some editors are assuming HTML input, they won't work (well) with non HTML input.
For example its kinda useless to use `html_prettify` after `html_textify` as text won't end up being pretty HTML.
For this reason editors like `css_select` are still producing valid HTML output.

There are probably more tasks out there that might be useful as editors.
Feel free to provide an issue for an editor idea or create a Pull Request with a new editor.

#### `css_flatten`

Replaces every matching HTML element with its child nodes and returns the HTML.
Instead of [`css_remove`](#css_remove) this does not remove all the child nodes below.

Examples:

```yaml
editors:
  - css_flatten: div
  - css_flatten: a[href^="#"] # flatten all local links away (starting with a #)
```

#### `css_remove`

Tries to remove every instance of matching HTML elements and returns the remaining HTML.
Opposite of [`css_select`](#css_select).
When the child nodes should be kept, use [`css_flatten`](#css_flatten).

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

#### `css_sort`

Sort elements matching to the given [CSS Selector](https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/Selectors).
Other elements not matching are kept.
Elements below different parents are sorted independently.

Basic example:

```html
<div><p>C</p><p>B</p></div>
<div><p>D</p><p>A</p></div>
```

with `p` as the selector will sort into this:

```html
<div><p>B</p><p>C</p></div>
<div><p>A</p><p>D</p></div>
```

Examples:

```yaml
editors:
  # Sort all articles
  - css_sort:
      selector: article
```

The above example sorts by the whole element ([`outerHTML`](https://developer.mozilla.org/en-US/docs/Web/API/Element/outerHTML)).
In order to sort by something specific for a given HTML element, editors can be used.

```yaml
editors:
  # Sort articles by their heading
  - css_sort:
      selector: article
      sort_by: # the specified editors are applied to every selected HTML element independently
        - css_select: h2
```

This might still sort in surprising ways as things like attributes are still included (`<h2 class="a">Z</h2>` is sorted before `<h2 class="z">A</h2>`).
Therefore, editors like [`html_textify`](#html_textify) or [`html_sanitize`](#html_sanitize) are likely a good idea to be used in `sort_by`.

Tip: [`debug_files`](#debug_files) can help you understand what is happening. But don't forget to remove it after you are done testing:

```yaml
editors:
  - css_sort:
      selector: article
      sort_by:
        - css_select: h2
        - html_sanitize
        - debug_files: /tmp/website-stalker/
```

You can also reverse the sorting:

```yaml
editors:
  - css_sort:
      selector: article
      reverse: true
```

#### `css_tag_replace`

Replace HTML tags matching a given CSS selector.

For example, the following config will replace all `h3` tags with `h2` tags.

```yaml
editors:
  - css_tag_replace:
      selector: h3
      replace: h2
```

```diff
 <html>
 <head></head>
 <body>
-  <h3 class="green">
+  <h2 class="green">
     Hello
-  </h3>
+  </h2>
   World
 </body>
 </html>
```

#### `debug_files`

This editor passes its input through without modifying it.
The content is written to a file in the given directory.
The filename is created from the current UNIX Timestamp.

This is neat when looking at steps in between editors is of interest.
Especially for editors like [RSS](#rss) which use editors per item this can be handy to look at the steps in between.

Warning: It's not recommended committing these files.
`debug_files` should be removed before when committing the config.
It might have unintended side effects or might spam your repository with many potentially large files.

Examples:

```yaml
editors:
  - debug_files: /tmp/website-stalker/
```

#### `html_markdownify`

Formats the input HTML as Markdown.

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
  # Remove all occurrences of that word
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
            - css_remove: "h2, article > a, div"
            - html_textify

  # Minimal working example
  - url: "https://edjopato.de/post/"
    editors:
      - rss: {}
```

## Alternatives

- [Website Changed Bot](https://github.com/EdJoPaTo/website-changed-bot) is a Telegram Bot which might potentially use this tool later on
- [bernaferrari/ChangeDetection](https://github.com/bernaferrari/ChangeDetection) is an Android app for this
- [dgtlmoon/changedetection.io](https://github.com/dgtlmoon/changedetection.io) can be self-hosted and configured via web interface
- [Feed me up, Scotty!](https://gitlab.com/vincenttunru/feed-me-up-scotty) creates RSS feeds from websites
- [htmlq](https://github.com/mgdm/htmlq) command line tool to format / select HTML (like `jq` for HTML)
- [urlwatch](https://thp.io/2008/urlwatch/)
