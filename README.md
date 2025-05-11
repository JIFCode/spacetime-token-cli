# Spacetime Account CLI

A command-line tool to manage SpacetimeDB authentication tokens by synchronizing them between a local `accounts.toml` file and the SpacetimeDB CLI's `cli.toml` configuration.

## Features

- **Set Token**: Updates the `spacetimedb_token` in `~/.config/spacetime/cli.toml` with a token from a named account in `accounts.toml`.
- **Save Token**: Saves the current `spacetimedb_token` from `~/.config/spacetime/cli.toml` to a new named account in `accounts.toml`. Errors if the account name already exists.
- **Create Account**: Initiates `spacetime logout` then `spacetime login`, and saves the new token to a named account in `accounts.toml`. Errors if the account name already exists.
- **List Accounts**: Lists all account names stored in `accounts.toml`.
- **Delete Account**: Removes a specified account from `accounts.toml`.
- **Reset Accounts**: Clears all accounts from `accounts.toml`.

## Configuration

The tool uses a standard configuration directory: `~/.config/spacetime-account/` (on Linux/macOS).
If this directory or the files within it do not exist, they will be created with default values when the tool is first run or when the `setup` command is called.

1.  **`config.toml`** (located in `~/.config/spacetime-account/config.toml`):
    This file configures the behavior of the tool. You can customize these settings using the `spacetime-account setup` command.

    ```toml
    # Configuration for the Spacetime Account CLI tool

    # Name of the TOML file storing user accounts
    accounts_filename = "accounts.toml"

    # Path to the SpacetimeDB CLI config directory, relative to the user's home directory
    cli_config_dir_from_home = ".config/spacetime"

    # Filename of the SpacetimeDB CLI configuration file
    cli_config_filename = "cli.toml"

    # Key for the token within the SpacetimeDB CLI configuration file
    cli_token_key = "spacetimedb_token"
    ```

2.  **`accounts.toml`** (located by default in `~/.config/spacetime-account/accounts.toml`; filename is configurable via `accounts_filename` in `config.toml`):
    This TOML file stores your named accounts and their corresponding tokens.
    Example:
    ```toml
    admin = "token_for_admin_user"
    dev_user = "token_for_dev_user"
    ```
    If this file doesn't exist when an operation requires it, it will be created (typically empty, or populated by `create` or `save`).

## Prerequisites

- Rust and Cargo installed.
- SpacetimeDB CLI installed (for `cli.toml` interaction).

## Build

Navigate to the project directory and run:

```bash
cargo build
```

The executable will be located at `target/debug/spacetime-account`.

## Installation

To make the `spacetime-account` command available system-wide (recommended):

```bash
cargo install --path .
```

Ensure that `~/.cargo/bin` is in your system's `PATH` environment variable. After installation, you can run commands like `spacetime-account list` or `spacetime-account setup` from any terminal location.

The configuration files (`config.toml` and `accounts.toml`) will be automatically managed in the `~/.config/spacetime-account/` directory. You do not need to manually copy them after installation.
If you run the tool for the first time after installation and these files don't exist, they (and the directory) will be created with default settings. You can then use `spacetime-account setup` to customize the configuration.

## Usage

After building and installing the tool (see 'Build' and 'Installation' sections above), you can run the tool directly using the `spacetime-account` command from any terminal location.

Use `spacetime-account help` to see a list of all commands and their descriptions.

### Commands

#### 1. `set` - Save/Update Account and Set Active

Saves a new account or updates an existing account's token in `accounts.toml`, and then sets this account's token as active in `cli.toml`.

```bash
spacetime-account set <ACCOUNT_NAME> <TOKEN>
```

Example:
To save a new token for "dev_user" (or update it if "dev_user" already exists) and make it active:

```bash
spacetime-account set dev_user "your_new_or_updated_dev_user_token_here"
```

This command always requires both a username and a token. It will update `spacetimedb_token` in `~/.config/spacetime/cli.toml`. If `cli.toml` or its parent directories do not exist, they will be created.

#### 2. `switch` - Switch Active Account

Looks up `<ACCOUNT_NAME>` in `accounts.toml` and updates `cli.toml` to use its token, making it the active account. This command is used to switch between already stored accounts.

```bash
spacetime-account switch <ACCOUNT_NAME>
```

Example:
To set an existing stored account "admin" as active:

```bash
spacetime-account switch admin
```

#### 3. `save` - Save Current Token

Saves the current token from `cli.toml` to `accounts.toml` under a new account name.
It will error if the chosen account name already exists in `accounts.toml`.

```bash
spacetime-account save <ACCOUNT_NAME>
```

Example:

```bash
spacetime-account save my_current_session
```

This reads the `spacetimedb_token` from `~/.config/spacetime/cli.toml` and saves it under the name "my_current_session" in `accounts.toml`. If the token is not found in `cli.toml`, or if "my_current_session" already exists as an account, an error will be reported.

#### 4. `create` - Create New Account via Login

Guides you through `spacetime logout` and then `spacetime login --server-issued-login local`, then saves the newly acquired token to `accounts.toml` (in the config directory) under the provided account name.
It will error if the chosen account name already exists in `accounts.toml` _before_ starting the logout/login process.

```bash
spacetime-account create <ACCOUNT_NAME>
```

Example:

```bash
spacetime-account create new_user
```

This command requires the `spacetime` CLI to be installed and in your PATH.

#### 5. `list` - List Accounts

Lists all account names currently stored in `accounts.toml`. Highlights the currently active account by appending " (current)" if its token matches the one in `cli.toml`.

```bash
spacetime-account list
```

Example:

```bash
spacetime-account list
```

#### 6. `delete` - Delete Account

Removes the specified account from `accounts.toml`.

```bash
spacetime-account delete <ACCOUNT_NAME>
```

Example:

```bash
spacetime-account delete old_user
```

If the account does not exist, it will report an error.

#### 7. `reset` - Reset Accounts

Clears all entries from `accounts.toml`, effectively resetting it to an empty state.

```bash
spacetime-account reset
```

Example:

```bash
spacetime-account reset
```

#### 8. `setup` - Interactive Configuration

Allows you to interactively set or update the configuration values for the tool, such as the names and locations of files it uses. These settings are stored in `~/.config/spacetime-account/config.toml`.

#### 9. `current` - Show Current Active Account

Displays the token currently active in `cli.toml` (masked for security, showing only the beginning and end). If this token is associated with an account name in `accounts.toml`, that account name is also displayed.

```bash
spacetime-account current
```

Example:

```bash
spacetime-account current
```
