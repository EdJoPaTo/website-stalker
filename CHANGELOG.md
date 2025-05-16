# Change log

All notable changes to this project will be documented in this file.

The format is based on [Keep a change log](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.26.1] - 2025-05-16

### Fixed

- `html_markdownify`: Error on empty output. Emtpy output is unlikely the expected behaviour.
- `html_url_canonicalize`: Don't error on unparsable URL. Only warn and accept the broken one.

## [0.26.0] - 2025-04-10

### Added

- Generate the JSON Schema for the `website-stalker.yaml` with `website-stalker json-schema`. Its also added to the Github Release for easy usage.
- new editor: `css_tag_replace`
- Support zstd response body decompression

## [0.25.1] - 2025-01-30

### Fixed

- Hint about an error in the end but dont create another error for hint.

## [0.25.0] - 2024-10-23

### Added

- new editor: `css_flatten`

## [0.24.2] - 2024-08-16

### Fixed

- Improve output on errors, especially HTTP request errors

## [0.24.1] - 2024-08-14

### Added

- support for CSS :has and :is (due to updated dependencies)

## [0.24.0] - 2024-06-13

### Added

- new editor: `css_sort`
- new editor: `debug_files`

### Fixed

- RSS: improve content_editors error output

## [0.23.0] - 2024-05-14

### Added

- Show used HTTP version in the output (`HTTP/1.1`, `HTTP/2.0`, …)
- `http1_only` option to force usage of `HTTP/1`

### Changed

- Move notifications from environment variables to CLI. Can still be configured via environment variables, but they have different names now. Check --help.
- Document `WEBSITE_STALKER_FROM` in `--help`. Also allows for `--from`
- RSS: remove website-stalker version from the generator field
- Improve error handling by instant panic or cleaner human error message
- Deprecate `init` sub-command. Its more transparent to use `git init && website-stalker example-config > website-stalker.yaml`
- Deprecate `check` sub-command. `run` also checks the config and additionally runs it when correct which most people probably need.

### Breaking Changes

- Environment variable names for notifications differ and can now also be provided via --flags. Check --help.
- Error on notification_template in config. Notification configuration changed and is likely not working anymore, so hard error over a warning.

## [0.22.0] - 2024-02-13

### Added

- Automatically generated man pages from the CLI definition

### Changed

- `rss` uses the first title / heading element as RSS title (was only title before)
- Improve error output message on editor error
- `json_prettify` uses tabs instead of spaces now for better accessibility and smaller file sizes (`html_prettify` does the same)
- Show warnings for deprecated field usage in notification mustache template
- Show warning on `rss` without title (neither from explicit configuration nor the input HTML)

### Fixed

- Correctly detect duplicate hosts for delays between them (to reduce load on the host)
- systemd service is `Type=oneshot` now and can no longer be installed. The timer is the relevant unit and not the service.

## [0.21.0] - 2023-09-05

### Changed

- Files are sorted into folders of their domains

## [0.20.0] - 2023-04-11

### Added

- new editor: `html_sanitize`
- `headers` site options to supply additional headers on requests
- `filename` option to override the automatically derived file base name from a URL
- Support URLs with IP addresses

### Changed

- Use git executable instead of libgit2
- Improve example-config
- write full words configuration and git repository instead of its short versions on stdout
- Include port in filename when specified

### Removed

- `check --rewrite-yaml` and `check --print-yaml`

## [0.19.0] - 2022-05-11

### Added

- `ignore_error` site option to only warn on pages that fail regularly
- Generate deb/rpm packages

### Changed

- systemd files are now meant for packages (no …/local/… anymore)

### Fixed

- CLI: correct autocompletion with `ValueHint`

## [0.18.1] - 2022-02-02

### Fixed

- css_remove: prevent removing wrong content

## [0.18.0] - 2022-02-01

### Added

- URL queries are now considered

### Changed

- html_prettify: format/sort class and style
