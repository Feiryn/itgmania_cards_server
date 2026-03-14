# ITGmania Cards Server

This project aims to replace ITGmania USB key profiles with NFC cards.

This project is for Linux only.

## How it works

### Web server

A web server is provided to manage the profiles.

The default port is `8000` and the default password is `admin`. Please take a look at the [configuration](#configuration) section to change these values.

Most of the web server code has been generated using Copilot because I suck at doing frontend.

### Unix socket server

A Unix socket server is provided for ITGmania to interact with this project.

The server can handle several commands, such as:
- `READ`: Read the currently inserted cards.
- `RESET <player>`: Remove the currently inserted card for the specified player (1 or 2).
- `ENABLE`: Allow the server to read the cards. This command is sent by ITGmania when the game is launched.
- `DISABLE`: Disable the server's ability to read the cards. This command is sent by ITGmania when the game is closed.

### NFC card reader

This project supports for now only custom arduino-based NFC card readers. The reader is connected to the server via a serial port and sends the card ID when a card is inserted. Please see the double PN532 reader on my github profile.

For more information about the implementation, or if you want to implement your own reader, please look at [./src/reader](./src/reader).

You can also insert cards manually using the web server when testing or if you don't have a card reader.

Please also note that cards can only be inserted when an ITGmania session is active.

Currently supported card types are Mifare and Felica.

## Installation

### Prerequisites

- Rust
- All prerequisites in https://github.com/ITGmania/ITGmania/tree/release/Build

### Build and run

#### This project

```bash
cargo run
```

or

```bash
cargo build --release
```

If you build the project in release mode, don't forget to copy the `config.toml`, `Rocket.toml` and `reader.toml` files to the same directory as the executable.

#### ITGmania

In the ITGmania sources, copy [MemoryCardDriverThreaded_Linux.cpp](./patch/MemoryCardDriverThreaded_Linux.cpp) to `src/arch/MemoryCard` and replace the existing file.

Then, build ITGmania (https://github.com/ITGmania/ITGmania/tree/release/Build).

In ITGmania's Preferences.ini, change the following values:

```
MemoryCardProfiles=1
MemoryCardUsbBusP1=1
MemoryCardUsbBusP2=2
```

### Configuration

The password hash is stored in `config.toml` at the same level as the executable.

The server configuration is stored in `Rocket.toml` at the same level as the executable.

The reader configuration is stored in `reader.toml` at the same level as the executable. If you don't have a reader, you can leave the type as `none`.

## Screenshots

![Login form](./images/login_form.png)

![Create account](./images/create_account.png)

![Accounts list](./images/accounts_list.png)