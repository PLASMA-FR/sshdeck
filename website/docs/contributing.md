# Contributing

SSHDeck welcomes contributions that keep the project local-first, safe, fast, and terminal-native.

## Development setup

```bash
git clone https://github.com/PLASMA-FR/sshdeck
cd sshdeck
cargo check --locked
cargo test --locked
cargo run --locked
```

## Website setup

```bash
npm install
npm run docs:dev
```

## Before opening a PR

Run:

```bash
cargo check --locked
cargo test --locked
npm run docs:build
```

Run `cargo fmt` and `cargo clippy` when those tools are available.

## Contribution areas

- Remote file transfer execution
- Safe remote editing
- Machine-readable remote listing
- Health command parsing
- Live tunnel process management
- Accessibility and terminal compatibility
- Documentation and screenshots
- Packaging and releases

## Safety expectations

Do not commit private SSH configs, keys, credentials, `.env` files, real server IPs, or generated `target/` output.
