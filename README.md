# Ethereum Accounts Manager for macOS

Ethereum Accounts Manager for macOS is a user-friendly application designed to simplify the management of Ethereum accounts directly from a Mac. With this tool, you can effortlessly create, manage, and interact with your Ethereum accounts securely.

## Features

- **Account Creation:** Easily generate new Ethereum accounts.
- **Account Management:** View and manage multiple Ethereum accounts.
- **Transaction Management:** Send and receive Ether.
- **Secure Storage:** Private keys are securely stored using macOS Keychain.
- **User-Friendly Interface:** Modern, intuitive UI designed for macOS.

## Installation

### Prerequisites

- macOS 10.14 or later
- Rust (for development)
- Ethereum client (e.g., Geth or Infura)

### Steps

1. Clone the repository:
    ```sh
    git clone https://github.com/zemse/gm.git
    cd gm
    ```

2. Install the binary:
    ```sh
    cargo install --path .
    ```

## Usage

### Creating an Account

```
gm acc create
```

### Listing all accounts

```
gm acc ls
```

### Sign Message

```
gm sm "hello"
```

The above command will sign the above message using EIP-191 standard. These signatures can be verified from any tools such as [Etherscan](https://etherscan.io/verifiedSignatures).

## Contributing

We welcome contributions! To contribute:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Make your changes and commit them (`git commit -m 'Add new feature'`).
4. Push to the branch (`git push origin feature-branch`).
5. Create a Pull Request.

Please ensure your code follows our [Code of Conduct](CODE_OF_CONDUCT.md) and [Contributing Guidelines](CONTRIBUTING.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.