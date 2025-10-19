![flowrs_logo](./image/README/1683789045509.png)

Flowrs is a TUI application for [Apache Airflow](https://airflow.apache.org/). It allows you to monitor, inspect and manage Airflow DAGs from the comforts of your terminal. It is build with the [ratatui](https://ratatui.rs/) library.

![flowrs demo](./vhs/flowrs.gif)

## Installation

You can install `flowrs` via Homebrew if you're on macOS / Linux / WSL2:

```
brew tap jvanbuel/flowrs
brew install flowrs
```

or by downloading the binary directly from GitHub:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jvanbuel/flowrs/releases/latest/download/flowrs-tui-installer.sh | sh
```

Alternatively, you can build `flowrs` from source with `cargo`:

```bash
cargo install flowrs-tui --locked
```

## Usage

### Managed Airflow services

The easiest way to user `flowrs` is with a managed Airflow service. The currently supported managed services are:

- [x] Conveyor
- [x] Amazon Managed Workflows for Apache Airflow (MWAA)
- [ ] Google Cloud Composer
- [x] Astronomer

To enable a managed service, run `flowrs config enable -m <service>`. This will add the configuration for the managed service to your configuration file, or prompt you for the necessary configuration details. On startup `flowrs` will then try to find and connect to all available managed service's Airflow instances.

Note that for Astronomer, you need to set the `ASTRO_API_TOKEN` environment variable with your Astronomer API token (Organization, Workspace or Deployment) to be able to connect to the service.

### Custom Airflow instances

If you're self-hosting an Airflow instance, or your favorite managed service is not yet supported, you can register an Airflow server instance with the `flowrs config add` command:

![flowrs config add demo](./vhs/add_config.gif)

This creates an entry in a `~/.flowrs` configuration file. If you have multiple Airflow servers configured, you can easily switch between them in `flowrs` configuration screen.

Only basic authentication and bearer token authentication are supported. When selecting the bearer token option, you can either provide a static token or a command that generates a token.
