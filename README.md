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

2. Install dependencies:
    ```sh
    cargo build
    ```

3. Run the application:
    ```sh
    cargo run
    ```

## Usage

### Creating an Account

1. Open the Ethereum Accounts Manager.
2. Click on "Create Account."
3. Follow the prompts to generate a new Ethereum account.

### Managing Accounts

- View your accounts in the "Accounts" tab.
- Select an account to view details and manage transactions.

### Sending Transactions

1. Select the account from which you want to send Ether.
2. Click on "Send Transaction."
3. Enter the recipient's address and the amount to send.
4. Confirm the transaction.

### Receiving Transactions

- Your Ethereum address is displayed in the "Receive" tab.
- Share this address to receive Ether.

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