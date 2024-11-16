![flowrs_logo](./image/README/1683789045509.png)

Flowrs is a TUI application for [Apache Airflow](https://airflow.apache.org/). It allows you to monitor, inspect and manage Airflow DAGs from the comforts of your terminal. It is build with the [ratatui](https://ratatui.rs/) library.

![flowrs demo](./vhs/flowrs.gif)

## Installation

You can install `flowrs` via Homebrew if you're on macOS / Linux / WSL2:

`brew install jvanbuel/flowrs`

or by downloading the binary directly from GitHub:

`curl -s ....`

Alternatively, you can build `flowrs` from source by cloning the repository and install the project with `cargo`:

```bash
git clone git@github.com:jvanbuel/flowrs.git
cargo install --path .
```

## Features

## Usage

You can register an Airflow server instance with the `flowrs config add` command:

![flowrs config add demo](./vhs/add_config.gif)

This creates an entry in a `~/.flowrs` configuration file. If you have multiple Airflow servers configured, you can easily switch between them in `flowrs` starting screen.

Only basic authentication and bearer token authentication are currently supported. When selecting the bearer token option, you can either provide a static token or a command that generates a token.

### Managed services

`flowrs` supports the following managed services:

- [x] Conveyor
- [ ] Google Cloud Composer
- [ ] Amazon Managed Workflows for Apache Airflow (MWAA)
- [ ] Astronomer

To enable a managed service, add it to the `managed_services` section in the configuration file, e.g.:

```toml
managed_services = ["Conveyor"]
```

`flowrs` will then on startup try to find and connect to all the Airflow instances that are managed by the specified service.
