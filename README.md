<!-- ![>gm](./gm-banner-white.png) -->

Terminal-based Ethereum accounts manager for devs.

![gm demo](./gm-demo.gif)

<!-- ![gm screenshot](./gm-screenshot.png) -->

Join [telegram group](https://t.me/+m2Lq_q_XqfcyN2Jl) for following this project.

## Features

- Secure store: securely store your keys in Apple Keychain (more locations are WIP)
- Address book: keep familiar accounts handy
- Light client: don't trust data blindly (powered by [Helios](https://github.com/a16z/helios))
- Walletconnect: connect to dapps
- EIP-1193 provider: avoid .env secrets in your scripts

### How `gm` is different?

Most wallets treat updates casually, some hardware wallet apps would force firmware upgrades, and software wallets silently update in the background. For a security critical software like a wallet, that's risky.

`gm` is designed to be reliable above all else:

- Streamlined releases: nightly for fast iteration and stable is hand-picked from an old nightly candidate.
- New integrations: Adding support to new external services means at least a new minor version, to prevent exposure.
- 5-year guarantee: Stable minor version must work long term, if due to any API failure it doesn't, it is considered as a bug.
- Long term support: all minor versions receive security patches and bug fixes.

This is an unusually strong commitment that should be in the wallet space, we hardly see it because it's challenging and most people being onboarded to crypto don't care initially.

## Installation

### Package managers

> TODO publish built binaries

### From source

Installation from source is highly recommended if you do not trust the pre-built binaries in the releases. However, building locally takes several minutes depending on your system, but once its done you're good to go.

```sh
# 1. Clone the repository
git clone https://github.com/zemse/gm.git

# 2. Go inside the binary crate
cd gm/bin

# 3. Build the project and install in your path
cargo install --path .
```

## Usage

Start the application by opening the terminal typing gm and then hitting enter (that's a vibe).

```
$ gm
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
