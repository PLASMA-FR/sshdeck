# Installation

SSHDeck is not published on crates.io yet. Use the source install path.

## From source

```bash
git clone https://github.com/PLASMA-FR/sshdeck
cd sshdeck
cargo install --path .
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

## Quick connect

```bash
sshdeck user@host
```

Quick connect launches your system `ssh` command after restoring the terminal.

## Development run

```bash
cargo run
cargo run -- doctor
cargo test
```

## Future package install

After a crates.io release exists, this will become available:

```bash
cargo install sshdeck
```

Do not use that command yet unless the package has been published.
