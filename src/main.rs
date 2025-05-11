use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, process::Command as StdCommand};
use toml_edit::{DocumentMut, Item};

const APP_DIR_NAME: &str = "spacetime-account"; // Changed for consistency
const DEFAULT_ACCOUNTS_FILENAME: &str = "accounts.toml";
const DEFAULT_CONFIG_FILENAME: &str = "config.toml";
const SPACETIME_CLI_COMMAND: &str = "spacetime";

#[derive(Debug, Deserialize, Serialize)]
struct AppSettings {
    accounts_filename: String,
    cli_config_dir_from_home: String,
    cli_config_filename: String,
    cli_token_key: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            accounts_filename: DEFAULT_ACCOUNTS_FILENAME.to_string(),
            cli_config_dir_from_home: ".config/spacetime".to_string(),
            cli_config_filename: "cli.toml".to_string(),
            cli_token_key: "spacetimedb_token".to_string(),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "spacetime-account",
    version = "0.1.0",
    about = "Manages SpacetimeDB tokens"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Saves/updates an account with a token and sets it active
    Set(SetArgs),
    /// Saves the current active token from cli.toml to a new account name
    Save(SaveArgs),
    /// Resets (clears) the accounts.toml file
    Reset,
    /// Creates a new account via 'spacetime login' and saves the token
    Create(CreateArgs),
    /// Lists all stored account names
    List,
    /// Deletes a stored account
    Delete(DeleteArgs),
    /// Interactive setup for configuration values
    Setup,
    /// Switches the active token to a stored account
    Switch(SwitchArgs),
    /// Displays the current active account name and token (masked)
    Current,
}

#[derive(Parser, Debug)]
struct SetArgs {
    /// The account name to save/update
    account_name: String,
    /// The token to associate with the account name
    token: String,
}

#[derive(Parser, Debug)]
struct SwitchArgs {
    /// The account name of the stored account to make active
    account_name: String,
}

#[derive(Parser, Debug)]
struct SaveArgs {
    /// The account name to save the current active token under
    account_name: String,
}

#[derive(Parser, Debug)]
struct CreateArgs {
    /// The account name for the new account
    account_name: String,
}

#[derive(Parser, Debug)]
struct DeleteArgs {
    /// The account name of the account to delete
    account_name: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct UserTokens(HashMap<String, String>);

fn get_app_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get user's config directory.")?
        .join(APP_DIR_NAME);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).with_context(|| {
            format!("Failed to create app config directory at {:?}", config_dir)
        })?;
        println!("Created application config directory at {:?}", config_dir);
    }
    Ok(config_dir)
}

fn load_app_settings() -> Result<AppSettings> {
    let app_config_dir = get_app_config_dir()?;
    let config_file_path = app_config_dir.join(DEFAULT_CONFIG_FILENAME);

    if !config_file_path.exists() {
        println!(
            "Configuration file not found at {:?}. Creating with default settings.",
            config_file_path
        );
        let default_settings = AppSettings::default();
        let toml_content = toml::to_string_pretty(&default_settings)
            .context("Failed to serialize default settings to TOML")?;
        fs::write(&config_file_path, toml_content)
            .with_context(|| format!("Failed to write default config to {:?}", config_file_path))?;
        return Ok(default_settings);
    }

    let content = fs::read_to_string(&config_file_path)
        .with_context(|| format!("Failed to read app config file at {:?}", config_file_path))?;
    toml::from_str(&content)
        .with_context(|| format!("Failed to parse app config file at {:?}", config_file_path))
}

fn write_app_settings(settings: &AppSettings) -> Result<()> {
    let app_config_dir = get_app_config_dir()?;
    let config_file_path = app_config_dir.join(DEFAULT_CONFIG_FILENAME);
    let toml_content =
        toml::to_string_pretty(settings).context("Failed to serialize app settings to TOML")?;
    fs::write(&config_file_path, toml_content)
        .with_context(|| format!("Failed to write app config to {:?}", config_file_path))?;
    println!("Configuration saved to {:?}", config_file_path);
    Ok(())
}

fn get_accounts_filepath(settings: &AppSettings) -> Result<PathBuf> {
    let app_config_dir = get_app_config_dir()?;
    Ok(app_config_dir.join(&settings.accounts_filename))
}

fn get_cli_toml_path(settings: &AppSettings) -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home_dir
        .join(&settings.cli_config_dir_from_home)
        .join(&settings.cli_config_filename))
}

fn read_accounts(settings: &AppSettings) -> Result<UserTokens> {
    let accounts_path = get_accounts_filepath(settings)?;
    if !accounts_path.exists() {
        match fs::write(&accounts_path, "") {
            Ok(_) => println!("Created empty {}.", settings.accounts_filename),
            Err(e) => {
                return Err(anyhow::Error::new(e).context(format!(
                    "Failed to create empty accounts file at {:?}",
                    accounts_path
                )));
            }
        }
        return Ok(UserTokens::default());
    }
    let content = fs::read_to_string(&accounts_path)
        .with_context(|| format!("Failed to read accounts file at {:?}", accounts_path))?;
    if content.trim().is_empty() {
        return Ok(UserTokens::default());
    }
    toml::from_str(&content).with_context(|| {
        format!(
            "Failed to parse accounts file at {:?}. Ensure it's valid TOML or empty.",
            accounts_path
        )
    })
}

fn write_accounts(settings: &AppSettings, accounts: &UserTokens) -> Result<()> {
    let accounts_path = get_accounts_filepath(settings)?;
    let content =
        toml::to_string_pretty(accounts).context("Failed to serialize accounts data to TOML")?;
    fs::write(&accounts_path, content)
        .with_context(|| format!("Failed to write accounts file at {:?}", accounts_path))?;
    println!("Successfully updated {}.", settings.accounts_filename);
    Ok(())
}

fn read_cli_toml(settings: &AppSettings) -> Result<DocumentMut> {
    let path = get_cli_toml_path(settings)?;
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read {} from {:?}",
            settings.cli_config_filename, path
        )
    })?;
    content.parse::<DocumentMut>().with_context(|| {
        format!(
            "Failed to parse {} from {:?}",
            settings.cli_config_filename, path
        )
    })
}

fn write_cli_toml(settings: &AppSettings, doc: &DocumentMut) -> Result<()> {
    let path = get_cli_toml_path(settings)?;
    fs::write(&path, doc.to_string()).with_context(|| {
        format!(
            "Failed to write {} to {:?}",
            settings.cli_config_filename, path
        )
    })?;
    println!("Successfully updated {}.", settings.cli_config_filename);
    Ok(())
}

fn run_external_command(command_name: &str, args: &[&str]) -> Result<()> {
    println!("Running: {} {}...", command_name, args.join(" "));
    let mut cmd = StdCommand::new(command_name);
    cmd.args(args);

    let status = cmd
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .with_context(|| {
            format!(
                "Failed to execute command: {}. Is '{}' in your PATH?",
                command_name, command_name
            )
        })?;

    if status.success() {
        println!(
            "Command '{} {}' executed successfully.",
            command_name,
            args.join(" ")
        );
        Ok(())
    } else {
        anyhow::bail!(
            "Command '{} {}' failed with status: {}",
            command_name,
            args.join(" "),
            status
        );
    }
}

fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        // Arbitrary length, too short to mask meaningfully
        return token.to_string();
    }
    format!("{}...{}", &token[..5], &token[token.len() - 5..])
}

fn main() -> Result<()> {
    let settings = load_app_settings().context("Failed to load application settings")?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Set(args) => {
            let mut accounts = read_accounts(&settings)?;
            accounts
                .0
                .insert(args.account_name.clone(), args.token.clone());
            write_accounts(&settings, &accounts)?;
            println!(
                "Account '{}' saved/updated in {}.",
                args.account_name, settings.accounts_filename
            );

            let cli_toml_path = get_cli_toml_path(&settings)?;
            let mut cli_toml = if cli_toml_path.exists() {
                read_cli_toml(&settings)?
            } else {
                if let Some(parent_dir) = cli_toml_path.parent() {
                    fs::create_dir_all(parent_dir)
                        .with_context(|| format!("Failed to create directory {:?}", parent_dir))?;
                }
                DocumentMut::new()
            };
            cli_toml[&settings.cli_token_key] = Item::Value(args.token.into());
            write_cli_toml(&settings, &cli_toml)?;
            println!(
                "Account '{}' also set as active token in {}.",
                args.account_name, settings.cli_config_filename
            );
        }
        Commands::Switch(args) => {
            let accounts = read_accounts(&settings)?;
            if let Some(token_from_accounts) = accounts.0.get(&args.account_name) {
                let cli_toml_path = get_cli_toml_path(&settings)?;
                let mut cli_toml = if cli_toml_path.exists() {
                    read_cli_toml(&settings)?
                } else {
                    if let Some(parent_dir) = cli_toml_path.parent() {
                        fs::create_dir_all(parent_dir).with_context(|| {
                            format!("Failed to create directory {:?}", parent_dir)
                        })?;
                    }
                    DocumentMut::new()
                };
                cli_toml[&settings.cli_token_key] = Item::Value(token_from_accounts.clone().into());
                write_cli_toml(&settings, &cli_toml)?;
                println!(
                    "Switched active token to account '{}' (from {}) in {}.",
                    args.account_name, settings.accounts_filename, settings.cli_config_filename
                );
            } else {
                println!(
                    "Account '{}' not found in {}. Cannot switch.",
                    args.account_name, settings.accounts_filename
                );
                println!("Available accounts: {:?}", accounts.0.keys());
                anyhow::bail!("Account not found in accounts file for switching.");
            }
        }
        Commands::Save(args) => {
            let cli_toml_path = get_cli_toml_path(&settings)?;
            if !cli_toml_path.exists() {
                anyhow::bail!(
                    "{} does not exist. Cannot save token.",
                    settings.cli_config_filename
                );
            }
            let cli_toml = read_cli_toml(&settings)?;

            let mut accounts = read_accounts(&settings)?;
            if accounts.0.contains_key(&args.account_name) {
                anyhow::bail!("Account '{}' already exists in {}. Use a different name or delete the existing one first.", args.account_name, settings.accounts_filename);
            }

            match cli_toml.get(&settings.cli_token_key) {
                Some(token_item) => {
                    if let Some(token_str) = token_item.as_str() {
                        accounts
                            .0
                            .insert(args.account_name.clone(), token_str.to_string());
                        write_accounts(&settings, &accounts)?;
                        println!(
                            "Saved current active token as '{}' in {}.",
                            args.account_name, settings.accounts_filename
                        );
                    } else {
                        anyhow::bail!(
                            "Token key '{}' in {} is not a string.",
                            settings.cli_token_key,
                            settings.cli_config_filename
                        );
                    }
                }
                None => {
                    anyhow::bail!(
                        "User is not logged in. Token key '{}' not found in {}.",
                        settings.cli_token_key,
                        settings.cli_config_filename
                    );
                }
            }
        }
        Commands::Reset => {
            let accounts = UserTokens::default();
            write_accounts(&settings, &accounts)?;
            println!("{} has been reset.", settings.accounts_filename);
        }
        Commands::Create(args) => {
            let mut accounts = read_accounts(&settings)?;
            if accounts.0.contains_key(&args.account_name) {
                anyhow::bail!(
                    "Account '{}' already exists in {}. Cannot create.",
                    args.account_name,
                    settings.accounts_filename
                );
            }

            run_external_command(SPACETIME_CLI_COMMAND, &["logout"])
                .context("Failed to logout from SpacetimeDB CLI.")?;

            println!(
                "Please follow the prompts from 'spacetime login --server-issued-login local'."
            );
            run_external_command(
                SPACETIME_CLI_COMMAND,
                &["login", "--server-issued-login", "local"],
            )
            .context("Failed during 'spacetime login --server-issued-login local'.")?;

            println!(
                "Login successful. Saving token as '{}'...",
                args.account_name
            );
            let cli_toml_path = get_cli_toml_path(&settings)?;
            if !cli_toml_path.exists() {
                anyhow::bail!(
                    "{} does not exist after login. Cannot save token.",
                    settings.cli_config_filename
                );
            }
            let cli_toml = read_cli_toml(&settings)?;
            match cli_toml.get(&settings.cli_token_key) {
                Some(token_item) => {
                    if let Some(token_str) = token_item.as_str() {
                        accounts
                            .0
                            .insert(args.account_name.clone(), token_str.to_string());
                        write_accounts(&settings, &accounts)?;
                        println!(
                            "Successfully created and saved account '{}' in {}.",
                            args.account_name, settings.accounts_filename
                        );
                    } else {
                        anyhow::bail!(
                            "Token key '{}' in {} is not a string after login.",
                            settings.cli_token_key,
                            settings.cli_config_filename
                        );
                    }
                }
                None => {
                    anyhow::bail!(
                        "Token key '{}' not found in {} after login.",
                        settings.cli_token_key,
                        settings.cli_config_filename
                    );
                }
            }
        }
        Commands::List => {
            let accounts = read_accounts(&settings)?;
            let mut active_token_opt: Option<String> = None;

            if let Ok(cli_toml_path) = get_cli_toml_path(&settings) {
                if cli_toml_path.exists() {
                    if let Ok(cli_toml_doc) = read_cli_toml(&settings) {
                        if let Some(token_item) = cli_toml_doc.get(&settings.cli_token_key) {
                            if let Some(token_str) = token_item.as_str() {
                                active_token_opt = Some(token_str.to_string());
                            }
                        }
                    }
                }
            }

            if accounts.0.is_empty() {
                println!("No accounts found in {}.", settings.accounts_filename);
            } else {
                println!("Available accounts in {}:", settings.accounts_filename);
                let mut sorted_account_names: Vec<_> = accounts.0.keys().collect();
                sorted_account_names.sort();

                for acc_name in sorted_account_names {
                    let mut display_name = format!("- {}", acc_name);
                    if let Some(ref active_token) = active_token_opt {
                        if let Some(user_token) = accounts.0.get(acc_name) {
                            if user_token == active_token {
                                display_name.push_str(" (current)");
                            }
                        }
                    }
                    println!("{}", display_name);
                }
            }
        }
        Commands::Current => {
            let cli_toml_path = get_cli_toml_path(&settings)?;
            if !cli_toml_path.exists() {
                println!(
                    "{} not found. No active token set.",
                    settings.cli_config_filename
                );
                return Ok(());
            }
            let cli_toml_doc = read_cli_toml(&settings)?;
            if let Some(token_item) = cli_toml_doc.get(&settings.cli_token_key) {
                if let Some(active_token_str) = token_item.as_str() {
                    let accounts = read_accounts(&settings)?;
                    let mut current_account_name: Option<String> = None;
                    for (acc_name, token) in accounts.0.iter() {
                        if token == active_token_str {
                            current_account_name = Some(acc_name.clone());
                            break;
                        }
                    }
                    if let Some(name) = current_account_name {
                        println!("Current active account: {}", name);
                    } else {
                        println!(
                            "Current active token is set, but not found under any account name in {}.",
                            settings.accounts_filename
                        );
                    }
                    println!("Active token: {}", mask_token(active_token_str));
                } else {
                    println!(
                        "Active token key '{}' in {} is not a string.",
                        settings.cli_token_key, settings.cli_config_filename
                    );
                }
            } else {
                println!(
                    "No active token (key '{}') found in {}.",
                    settings.cli_token_key, settings.cli_config_filename
                );
            }
        }
        Commands::Delete(args) => {
            let mut accounts = read_accounts(&settings)?;
            if accounts.0.remove(&args.account_name).is_some() {
                write_accounts(&settings, &accounts)?;
                println!(
                    "Account '{}' deleted from {}.",
                    args.account_name, settings.accounts_filename
                );
            } else {
                println!(
                    "Account '{}' not found in {}. Nothing to delete.",
                    args.account_name, settings.accounts_filename
                );
                anyhow::bail!("Account not found for deletion.");
            }
        }
        Commands::Setup => {
            let mut current_settings = load_app_settings().unwrap_or_else(|e| {
                println!(
                    "Warning: Could not load existing settings ({}). Using defaults.",
                    e
                );
                AppSettings::default()
            });

            println!("Current configuration (leave blank to keep current value):");

            let mut input = String::new();
            println!(
                "Accounts filename [{}]: ",
                current_settings.accounts_filename
            );
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().is_empty() {
                current_settings.accounts_filename = input.trim().to_string();
            }
            input.clear();

            println!(
                "SpacetimeDB CLI config directory (from home) [{}]: ",
                current_settings.cli_config_dir_from_home
            );
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().is_empty() {
                current_settings.cli_config_dir_from_home = input.trim().to_string();
            }
            input.clear();

            println!(
                "SpacetimeDB CLI config filename [{}]: ",
                current_settings.cli_config_filename
            );
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().is_empty() {
                current_settings.cli_config_filename = input.trim().to_string();
            }
            input.clear();

            println!(
                "SpacetimeDB CLI token key [{}]: ",
                current_settings.cli_token_key
            );
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().is_empty() {
                current_settings.cli_token_key = input.trim().to_string();
            }

            write_app_settings(&current_settings)?;
        }
    }

    Ok(())
}
