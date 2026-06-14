# Installation

SSHDeck is not published on crates.io yet. Use the source install path.

## From source

```bash
git clone https://github.com/PLASMA-FR/sshdeck
cd sshdeck
bash scripts/install.sh
```

The script installs the current checkout with `cargo install --locked --path <checkout>`.

For a one-command install from GitHub:

```bash
curl -fsSL https://raw.githubusercontent.com/PLASMA-FR/sshdeck/main/scripts/install.sh | bash
```

Direct Cargo install from a checkout also works:

```bash
cargo install --locked --path .
```

## Run

```bash
sshdeck
```

## Doctor

```bash
sshdeck doctor
```

## Import

```bash
sshdeck import
```

`import` parses `~/.ssh/config`, prints the number of parseable hosts, and ensures the SSHDeck app config exists. It does not copy or rewrite your OpenSSH config. The TUI also reads `~/.ssh/config` at startup.

## Quick connect

```bash
sshdeck user@host
```

Quick connect launches your system `ssh` command after restoring the terminal.

## CLI options

```text
sshdeck [OPTIONS] [TARGET] [COMMAND]

Commands:
  doctor          check local OpenSSH tools, config, terminal, and defaults
  import          parse ~/.ssh/config and ensure app config exists

Options:
  --config <PATH>       use a specific SSHDeck config.toml
  --theme <THEME>       override the configured theme for this run
  --no-animations       disable animations for this run
  --ascii               force ASCII-friendly rendering
  --mouse               force mouse capture on for this run
  --no-mouse            disable mouse capture for this run
  --quick <TARGET>      quick-connect with system ssh
  -h, --help            print help
  -V, --version         print version
```

`TARGET` and `--quick <TARGET>` both bypass the TUI and run system `ssh -- <TARGET>`.

## Development run

```bash
cargo run --locked
cargo run --locked -- doctor
cargo test --locked
```

## Future package install

After a crates.io release exists, this will become available:

```bash
cargo install sshdeck
```

Do not use that command yet unless the package has been published.
