# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
