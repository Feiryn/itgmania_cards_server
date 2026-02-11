# ITGMania Cards Server

This project aims to replace the ITGMania USB key profiles by NFC cards.

This project is Linux only.

## How it works

### Web server

A web server is exposed to manage the profiles.

The default port is `8000` and the default password is `admin`. Please take a look at the [configuration](#configuration) section to change these values.

Most of the web server code has been generated using Copilot because I suck at doing front end.

Here are some screenshots of the web server:

![Login form](./images/login_form.png)

![Create account](./images/create_account.png)

![Accounts list](./images/accounts_list.png)

### Unix socket server

A Unix socket server is exposed for ITGMania to interact with this project.

The server can handle several commands, such as:
- `READ`: Read the currently inserted cards.
- `RESET <player>`: Remove the currently inserted card for the specified player (1 or 2).
- `ENABLE`: Allow the server to read the cards. This command is sent by ITGMania when the game is launched.
- `DISABLE`: Disable the server ability to read the cards. This command is sent by ITGMania when the game is closed.

### NFC card reader

This is left to do, but the idea is to use two NFC card readers, one for each player, to read the cards and update the inserted cards in the server.

For now, the cards can be manually inserted using the web server using buttons on the accounts list page.

The buttons are available only if the server is enabled, which means that ITGMania is launched and has sent the `ENABLE` command.

## Installation

### Prerequisites

- Rust
- All prerequisites in https://github.com/itgmania/itgmania/tree/release/Build

### Build and run

#### This project

```bash
cargo run
```

or

```bash
cargo build --release
```

If you build the project in release mode, don't forget to copy the `config.toml` and `Rocket.toml` files to the same directory as the executable.

#### ITGMania

In the itgmania sources, copy [MemoryCardDriverThreaded_Linux.cpp](./patch/MemoryCardDriverThreaded_Linux.cpp) to `src/arch/MemoryCard` and replace the existing file.

Then, build ITGMania (https://github.com/itgmania/itgmania/tree/release/Build).

### Configuration

The password hash is stored in `config.toml` at the same level as the executable.

The server configuration is stored in `Rocket.toml` at the same level as the executable.
