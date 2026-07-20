# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.2](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.12.1...flowrs-config-v0.12.2) - 2026-07-20

### Other

- implement Debug for public types; enable missing_debug_implementations
- convert #[allow] to #[expect] with reasons; enable allow_attributes_without_reason ([#677](https://github.com/jvanbuel/flowrs/pull/677))

## [0.12.1](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.12.0...flowrs-config-v0.12.1) - 2026-07-19

### Fixed

- *(config)* surface a clean error when the home directory is unavailable ([#672](https://github.com/jvanbuel/flowrs/pull/672))
- *(config)* make writes atomic and stop swallowing read errors ([#669](https://github.com/jvanbuel/flowrs/pull/669))

### Other

- centralize lints in [workspace.lints] and lint the airflow crate ([#674](https://github.com/jvanbuel/flowrs/pull/674))

## [0.12.0](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.7...flowrs-config-v0.12.0) - 2026-07-10

### Other

- updated the following local packages: flowrs-airflow

## [0.11.7](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.6...flowrs-config-v0.11.7) - 2026-07-09

### Other

- updated the following local packages: flowrs-airflow

## [0.11.6](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.5...flowrs-config-v0.11.6) - 2026-07-01

### Other

- updated the following local packages: flowrs-airflow

## [0.11.5](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.4...flowrs-config-v0.11.5) - 2026-06-18

### Other

- update Cargo.toml dependencies

## [0.11.4](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.3...flowrs-config-v0.11.4) - 2026-05-25

### Other

- update Cargo.toml dependencies

## [0.11.3](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.2...flowrs-config-v0.11.3) - 2026-05-15

### Other

- updated the following local packages: flowrs-airflow

## [0.11.2](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.1...flowrs-config-v0.11.2) - 2026-05-10

### Other

- updated the following local packages: flowrs-airflow

## [0.11.1](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.11.0...flowrs-config-v0.11.1) - 2026-04-23

### Added

- Add insecure SSL connection option ([#632](https://github.com/jvanbuel/flowrs/pull/632))

## [0.11.0](https://github.com/jvanbuel/flowrs/compare/flowrs-config-v0.10.1...flowrs-config-v0.11.0) - 2026-03-14

### Added

- add dark/light theme support with auto-detection
