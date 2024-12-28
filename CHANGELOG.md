# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6](https://github.com/jvanbuel/flowrs/compare/v0.1.5...v0.1.6) - 2024-12-28

### Fixed

- do not handle non-tick events when loading
- reduce gif sizes

### Other

- Merge pull request [#290](https://github.com/jvanbuel/flowrs/pull/290) from jvanbuel/dependabot/cargo/serde_json-1.0.134
- Merge pull request [#291](https://github.com/jvanbuel/flowrs/pull/291) from jvanbuel/dependabot/cargo/anyhow-1.0.95
- *(deps)* bump env_logger from 0.11.5 to 0.11.6
- update vhs gif
- Merge pull request [#286](https://github.com/jvanbuel/flowrs/pull/286) from jvanbuel/dependabot/cargo/chrono-0.4.39
- *(deps)* bump chrono from 0.4.38 to 0.4.39
- show cursor position
- refactor filter into widget
- Create CODE_OF_CONDUCT.md
- Update issue templates
- Create LICENSE
- refactor command help with lazy static

## [0.1.5](https://github.com/jvanbuel/flowrs/compare/v0.1.4...v0.1.5) - 2024-12-08

### Added

- open browser to endpoint when pressing 'o'
- use rounded corners for Blocks
- use anyhow instead of custom errors
- add cli command to enable/disable managed services
- implement gg key binding for jumping to top

### Fixed

- UI issues and filter not consuming events
- spinner UI hangs when pressing escape
- remove trigger command from taskinstances

### Other

- replace lazy_static with stdlib LazyLock
- persist active config

## [0.1.4](https://github.com/jvanbuel/flowrs/compare/v0.1.3...v0.1.4) - 2024-11-29

### Fixed

- consume key when commands are exited

### Other

- *(deps)* bump rustls from 0.23.17 to 0.23.18
- *(deps)* bump url from 2.5.3 to 2.5.4
- update installation instructions in README.md

## [0.1.3](https://github.com/jvanbuel/flowrs/compare/v0.1.2...v0.1.3) - 2024-11-17

### Other

- use flowrs as homebrew formula name

## [0.1.2](https://github.com/jvanbuel/flowrs/compare/v0.1.1...v0.1.2) - 2024-11-17

### Other

- use git-lfs for gifs

## [0.1.1](https://github.com/jvanbuel/flowrs/compare/v0.1.0...v0.1.1) - 2024-11-16

### Other

- first release
