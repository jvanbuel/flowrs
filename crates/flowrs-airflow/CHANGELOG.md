# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.7](https://github.com/jvanbuel/flowrs/compare/flowrs-airflow-v0.10.6...flowrs-airflow-v0.10.7) - 2026-07-01

### Added

- show scheduled and queued phases in the Gantt chart

## [0.10.6](https://github.com/jvanbuel/flowrs/compare/flowrs-airflow-v0.10.5...flowrs-airflow-v0.10.6) - 2026-06-18

### Other

- Merge pull request #651 from jvanbuel/dependabot/cargo/aws-config-1.8.18
- *(deps)* bump google-cloud-auth from 1.10.0 to 1.12.0
- *(deps)* bump aws-sdk-mwaa from 1.106.0 to 1.108.0

## [0.10.5](https://github.com/jvanbuel/flowrs/compare/flowrs-airflow-v0.10.4...flowrs-airflow-v0.10.5) - 2026-05-25

### Other

- *(deps)* bump aws-config from 1.8.16 to 1.8.17
- *(deps)* bump aws-sdk-mwaa from 1.105.0 to 1.106.0

## [0.10.4](https://github.com/jvanbuel/flowrs/compare/flowrs-airflow-v0.10.3...flowrs-airflow-v0.10.4) - 2026-05-15

### Other

- update Cargo.toml dependencies

## [0.10.3](https://github.com/jvanbuel/flowrs/compare/flowrs-airflow-v0.10.2...flowrs-airflow-v0.10.3) - 2026-05-10

### Fixed

- Handle missing display_name fields and improve JSON parse error reporting for Airflow v2.8.1 ([#628](https://github.com/jvanbuel/flowrs/pull/628))

## [0.10.2](https://github.com/jvanbuel/flowrs/compare/flowrs-airflow-v0.10.1...flowrs-airflow-v0.10.2) - 2026-04-23

### Added

- Add insecure SSL connection option ([#632](https://github.com/jvanbuel/flowrs/pull/632))

### Other

- *(deps)* bump google-cloud-auth from 1.7.0 to 1.8.0 ([#625](https://github.com/jvanbuel/flowrs/pull/625))
