# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
