# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.12](https://github.com/jvanbuel/flowrs/compare/v0.1.11...v0.1.12) - 2025-04-21

### Fixed

- use own username in homebrew publishing step

## [0.1.11](https://github.com/jvanbuel/flowrs/compare/v0.1.10...v0.1.11) - 2025-04-21

### Fixed

- release with newer ubuntu version

## [0.1.9](https://github.com/jvanbuel/flowrs/compare/v0.1.8...v0.1.9) - 2025-04-21

### Added

- remove dependency on jq for getting conveyor tokens

### Fixed

- remove leading slash so conveyor URLs for DagRuns and Task Instances are correctly formed

### Other

- *(deps)* bump clap from 4.5.36 to 4.5.37
- Merge pull request #349 from jvanbuel/dependabot/cargo/clap-4.5.36
- *(deps)* bump anyhow from 1.0.97 to 1.0.98
- Merge pull request #346 from jvanbuel/dependabot/cargo/crossterm-0.29.0
- Merge pull request #345 from jvanbuel/dependabot/cargo/env_logger-0.11.8
- Merge pull request #344 from jvanbuel/dependabot/cargo/openssl-0.10.72
- Merge pull request #347 from jvanbuel/dependabot/cargo/tokio-1.44.2
- *(deps)* bump log from 0.4.26 to 0.4.27
- Merge pull request #337 from jvanbuel/dependabot/cargo/env_logger-0.11.7
- Merge pull request #339 from jvanbuel/dependabot/cargo/webbrowser-1.0.4
- Merge pull request #341 from jvanbuel/dependabot/cargo/tokio-1.44.1
- Merge pull request #342 from jvanbuel/dependabot/cargo/time-0.3.41
- *(deps)* bump reqwest from 0.12.12 to 0.12.15
- *(deps)* bump serde from 1.0.218 to 1.0.219
- Merge pull request #332 from jvanbuel/dependabot/cargo/ring-0.17.13
- Merge pull request #334 from jvanbuel/dependabot/cargo/indoc-2.0.6
- Merge pull request #335 from jvanbuel/dependabot/cargo/serde_json-1.0.140
- *(deps)* bump mockito from 1.6.1 to 1.7.0
- Merge pull request #328 from jvanbuel/dependabot/cargo/clap-4.5.31
- Merge pull request #329 from jvanbuel/dependabot/cargo/chrono-0.4.40
- Merge pull request #330 from jvanbuel/dependabot/cargo/rstest-0.25.0
- *(deps)* bump anyhow from 1.0.96 to 1.0.97
- Merge pull request #322 from jvanbuel/dependabot/cargo/log-0.4.26
- *(deps)* bump log from 0.4.25 to 0.4.26
- Merge pull request #324 from jvanbuel/dependabot/cargo/anyhow-1.0.96
- Merge pull request #325 from jvanbuel/dependabot/cargo/serde_json-1.0.139
- *(deps)* bump clap from 4.5.29 to 4.5.30
- update dependencies
- Merge pull request #320 from jvanbuel/dependabot/cargo/clap-4.5.29
- *(deps)* bump clap from 4.5.28 to 4.5.29
- Merge pull request #318 from jvanbuel/dependabot/cargo/openssl-0.10.70
- *(deps)* bump openssl from 0.10.69 to 0.10.70
- Merge pull request #315 from jvanbuel/dependabot/cargo/toml-0.8.20
- *(deps)* bump toml from 0.8.19 to 0.8.20

## [0.1.8](https://github.com/jvanbuel/flowrs/compare/v0.1.7...v0.1.8) - 2025-01-31

### Fixed

- allow using Left/Right keys for switching between TaskInstance logs

### Other

- Make state field optional so running DagRuns can be deserialized correctly

## [0.1.7](https://github.com/jvanbuel/flowrs/compare/v0.1.6...v0.1.7) - 2025-01-20

### Fixed

- persist last selected config state when switching between configurations

### Other

- default to empty config if no config file exists
- Merge pull request [#304](https://github.com/jvanbuel/flowrs/pull/304) from jvanbuel/dependabot/cargo/log-0.4.25
- *(deps)* bump serde_json from 1.0.135 to 1.0.137
- Merge pull request [#299](https://github.com/jvanbuel/flowrs/pull/299) from jvanbuel/dependabot/cargo/dirs-6.0.0
- Merge pull request [#300](https://github.com/jvanbuel/flowrs/pull/300) from jvanbuel/dependabot/cargo/tokio-1.43.0
- Merge pull request [#301](https://github.com/jvanbuel/flowrs/pull/301) from jvanbuel/dependabot/cargo/serde_json-1.0.135
- *(deps)* bump clap from 4.5.23 to 4.5.26
- *(deps)* bump reqwest from 0.12.11 to 0.12.12
- *(deps)* bump rstest from 0.23.0 to 0.24.0
- update README with managed service configuration

## [0.1.6](https://github.com/jvanbuel/flowrs/compare/v0.1.5...v0.1.6) - 2024-12-28

### Fixed

- do not handle non-tick events when loading
- reduce gif sizes

### Added
- Show cursor position
- Create CODE_OF_CONDUCT.md
- Create LICENSE

### Changed
- Update vhs gif
- Refactor filter into widget
- Refactor command help with lazy static
- Update issue templates

### Dependencies
- Bump chrono from 0.4.38 to 0.4.39
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
