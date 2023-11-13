# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Improve error output message on editor error
- `json_prettify` uses tabs instead of spaces now for better accessibility and smaller file sizes (`html_prettify` does the same)

### Fixed

- Correctly detect duplicate hosts for delays between them (to reduce load on the host)

## [0.21.0] - 2023-09-05

### Changed

- Files are sorted into folders of their domains

## [0.20.0] - 2023-04-11

### Added

- new editor: `html_sanitize`
- `headers` site options to supply additional headers on requests
- `filename` option to override the automatically derived file base name from an url
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
