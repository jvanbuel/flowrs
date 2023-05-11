![1683789045509](image/README/1683789045509.png)

Flowrs is a TUI application for Apache Airflow. It allows you to monitor, inspect and trigger Airflow DAGs from the comforts of your terminal. It is written with the Rust [ratatui](https://github.com/tui-rs-revival/ratatui) framework, using `crossterm` as the backend.

## Installation

You can install `flowrs` via Homebrew if you're on macOS / Linux / WSL2:

`brew install jvanbuel/flowrs`

or by downloading the binary directly from GitHub:

`curl -s ....`

Alternatively, you can build `flowrs` from source by cloning the repository and install the project with `cargo`:

```bash
git clone git@github.com:jvanbuel/flowrs.git
cargo install . 
```

## Usage

You can register an Airflow server instance with the `flowrs config add` command:

TODO: Add example of prompt

This creates an entry in a `~/.flowrs` configuration file. If you have multiple Airflow servers configured, you can easily switch between them in `flowrs` starting screen.

Currently only basic authentication and token authenication (via third-party OAuth2 plugins, e.g. `apache-airflow-providers-google`) are supported
