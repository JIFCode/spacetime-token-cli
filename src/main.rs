use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, process::Command as StdCommand};
use toml_edit::{DocumentMut, Item};

const APP_DIR_NAME: &str = "spacetime-token"; // Renamed
const DEFAULT_PROFILES_FILENAME: &str = "profiles.toml"; // Renamed
const DEFAULT_CONFIG_FILENAME: &str = "config.toml";
const SPACETIME_CLI_COMMAND: &str = "spacetime";

#[derive(Debug, Deserialize, Serialize)]
struct AppSettings {
    profiles_filename: String, // Renamed
    cli_config_dir_from_home: String,
    cli_config_filename: String,
    cli_token_key: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            profiles_filename: DEFAULT_PROFILES_FILENAME.to_string(), // Renamed
            cli_config_dir_from_home: ".config/spacetime".to_string(),
            cli_config_filename: "cli.toml".to_string(),
            cli_token_key: "spacetimedb_token".to_string(),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "spacetime-token", // Renamed
    version = "0.1.0",
    about = "Manages SpacetimeDB tokens via profiles" // Updated about
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Saves/updates a profile with a token and sets it active
    Set(SetArgs),
    /// Saves the current active token from cli.toml to a new profile name
    Save(SaveArgs),
    /// Resets (clears) the profiles.toml file
    Reset,
    /// Creates a new profile via 'spacetime login' and saves the token
    Create(CreateArgs),
    /// Lists all stored profile names
    List,
    /// Deletes a stored profile
    Delete(DeleteArgs),
    /// Interactive setup for configuration values
    Setup,
    /// Switches the active token to a stored profile
    Switch(SwitchArgs),
    /// Displays the current active profile name and token (masked)
    Current,
    /// Switches to the admin profile
    Admin,
}

#[derive(Parser, Debug)]
struct SetArgs {
    /// The profile name to save/update
    profile_name: String, // Renamed
    /// The token to associate with the profile name
    token: String,
}

#[derive(Parser, Debug)]
struct SwitchArgs {
    /// The profile name of the stored profile to make active (optional)
    profile_name: Option<String>, // Renamed
}

#[derive(Parser, Debug)]
struct SaveArgs {
    /// The profile name to save the current active token under
    profile_name: String, // Renamed
}

#[derive(Parser, Debug)]
struct CreateArgs {
    /// The profile name for the new profile
    profile_name: String, // Renamed
}

#[derive(Parser, Debug)]
struct DeleteArgs {
    /// The profile name of the profile to delete
    profile_name: String, // Renamed
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct UserProfiles(HashMap<String, String>); // Renamed

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

fn get_profiles_filepath(settings: &AppSettings) -> Result<PathBuf> {
    // Renamed function
    let app_config_dir = get_app_config_dir()?;
    Ok(app_config_dir.join(&settings.profiles_filename)) // Renamed field
}

fn get_cli_toml_path(settings: &AppSettings) -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home_dir
        .join(&settings.cli_config_dir_from_home)
        .join(&settings.cli_config_filename))
}

fn read_profiles(settings: &AppSettings) -> Result<UserProfiles> {
    // Renamed function and return type
    let profiles_path = get_profiles_filepath(settings)?; // Renamed variable
    if !profiles_path.exists() {
        match fs::write(&profiles_path, "") {
            // Renamed variable
            Ok(_) => println!("Created empty {}.", settings.profiles_filename), // Renamed field
            Err(e) => {
                return Err(anyhow::Error::new(e).context(format!(
                    "Failed to create empty profiles file at {:?}", // Renamed
                    profiles_path                                   // Renamed variable
                )));
            }
        }
        return Ok(UserProfiles::default()); // Renamed type
    }
    let content = fs::read_to_string(&profiles_path) // Renamed variable
        .with_context(|| format!("Failed to read profiles file at {:?}", profiles_path))?; // Renamed
    if content.trim().is_empty() {
        return Ok(UserProfiles::default()); // Renamed type
    }
    toml::from_str(&content).with_context(|| {
        format!(
            "Failed to parse profiles file at {:?}. Ensure it's valid TOML or empty.", // Renamed
            profiles_path // Renamed variable
        )
    })
}

fn write_profiles(settings: &AppSettings, profiles: &UserProfiles) -> Result<()> {
    // Renamed function and param
    let profiles_path = get_profiles_filepath(settings)?; // Renamed variable
    let content =
        toml::to_string_pretty(profiles).context("Failed to serialize profiles data to TOML")?; // Renamed
    fs::write(&profiles_path, content) // Renamed variable
        .with_context(|| format!("Failed to write profiles file at {:?}", profiles_path))?; // Renamed
    println!("Successfully updated {}.", settings.profiles_filename); // Renamed field
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
            let mut profiles = read_profiles(&settings)?; // Renamed
            profiles // Renamed
                .0
                .insert(args.profile_name.clone(), args.token.clone()); // Renamed
            write_profiles(&settings, &profiles)?; // Renamed
            println!(
                "Profile '{}' saved/updated in {}.", // Renamed
                args.profile_name,
                settings.profiles_filename // Renamed
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
                "Profile '{}' also set as active token in {}.", // Renamed
                args.profile_name,
                settings.cli_config_filename // Renamed
            );
        }
        Commands::Switch(args) => {
            let profiles = read_profiles(&settings)?; // Renamed
            let profile_name_to_switch = match args.profile_name {
                // Renamed
                Some(name) => name,
                None => {
                    if profiles.0.is_empty() {
                        // Renamed
                        println!(
                            "No profiles found in {}. Cannot switch.", // Renamed
                            settings.profiles_filename                 // Renamed
                        );
                        anyhow::bail!("No profiles available to switch."); // Renamed
                    }
                    let profile_names: Vec<&String> = profiles.0.keys().collect(); // Renamed
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select profile to switch to") // Renamed
                        .items(&profile_names) // Renamed
                        .default(0)
                        .interact_opt()?
                        .context("No profile selected or selection cancelled.")?; // Renamed

                    profile_names[selection].clone() // Renamed
                }
            };

            if let Some(token_from_profiles) = profiles.0.get(&profile_name_to_switch) {
                // Renamed
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
                cli_toml[&settings.cli_token_key] = Item::Value(token_from_profiles.clone().into()); // Renamed
                write_cli_toml(&settings, &cli_toml)?;
                println!(
                    "Switched active token to profile '{}' (from {}) in {}.", // Renamed
                    profile_name_to_switch,                                   // Renamed
                    settings.profiles_filename,                               // Renamed
                    settings.cli_config_filename
                );
            } else {
                println!(
                    "Profile '{}' not found in {}. Cannot switch.", // Renamed
                    profile_name_to_switch,
                    settings.profiles_filename // Renamed
                );
                println!("Available profiles: {:?}", profiles.0.keys()); // Renamed
                anyhow::bail!("Profile not found in profiles file for switching.");
                // Renamed
            }
        }
        Commands::Admin => {
            let admin_profile_name = "admin".to_string(); // Renamed
            let profiles = read_profiles(&settings)?; // Renamed
            if let Some(token_from_profiles) = profiles.0.get(&admin_profile_name) {
                // Renamed
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
                cli_toml[&settings.cli_token_key] = Item::Value(token_from_profiles.clone().into()); // Renamed
                write_cli_toml(&settings, &cli_toml)?;
                println!(
                    "Switched active token to ADMIN profile '{}' (from {}) in {}.", // Renamed
                    admin_profile_name,
                    settings.profiles_filename,
                    settings.cli_config_filename // Renamed
                );
            } else {
                println!(
                    "ADMIN profile ('{}') not found in {}. Cannot switch.", // Renamed
                    admin_profile_name,
                    settings.profiles_filename // Renamed
                );
                println!("Ensure a profile named 'admin' exists with a valid token."); // Renamed
                anyhow::bail!("Admin profile not found."); // Renamed
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

            let mut profiles = read_profiles(&settings)?; // Renamed
            if profiles.0.contains_key(&args.profile_name) {
                // Renamed
                anyhow::bail!("Profile '{}' already exists in {}. Use a different name or delete the existing one first.", args.profile_name, settings.profiles_filename);
                // Renamed
            }

            match cli_toml.get(&settings.cli_token_key) {
                Some(token_item) => {
                    if let Some(token_str) = token_item.as_str() {
                        profiles // Renamed
                            .0
                            .insert(args.profile_name.clone(), token_str.to_string()); // Renamed
                        write_profiles(&settings, &profiles)?; // Renamed
                        println!(
                            "Saved current active token as '{}' in {}.", // Renamed
                            args.profile_name,
                            settings.profiles_filename // Renamed
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
            let profiles = UserProfiles::default(); // Renamed
            write_profiles(&settings, &profiles)?; // Renamed
            println!("{} has been reset.", settings.profiles_filename); // Renamed
        }
        Commands::Create(args) => {
            let mut profiles = read_profiles(&settings)?; // Renamed
            if profiles.0.contains_key(&args.profile_name) {
                // Renamed
                anyhow::bail!(
                    "Profile '{}' already exists in {}. Cannot create.", // Renamed
                    args.profile_name,                                   // Renamed
                    settings.profiles_filename                           // Renamed
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
                args.profile_name // Renamed
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
                        profiles // Renamed
                            .0
                            .insert(args.profile_name.clone(), token_str.to_string()); // Renamed
                        write_profiles(&settings, &profiles)?; // Renamed
                        println!(
                            "Successfully created and saved profile '{}' in {}.", // Renamed
                            args.profile_name,
                            settings.profiles_filename // Renamed
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
            let profiles = read_profiles(&settings)?; // Renamed
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

            if profiles.0.is_empty() {
                // Renamed
                println!("No profiles found in {}.", settings.profiles_filename);
            // Renamed
            } else {
                println!("Available profiles in {}:", settings.profiles_filename); // Renamed
                let mut sorted_profile_names: Vec<_> = profiles.0.keys().collect(); // Renamed
                sorted_profile_names.sort(); // Renamed

                for profile_name in sorted_profile_names {
                    // Renamed
                    let mut display_name = format!("- {}", profile_name); // Renamed
                    if let Some(ref active_token) = active_token_opt {
                        if let Some(user_token) = profiles.0.get(profile_name) {
                            // Renamed
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
                    let profiles = read_profiles(&settings)?; // Renamed
                    let mut current_profile_name: Option<String> = None; // Renamed
                    for (profile_name, token) in profiles.0.iter() {
                        // Renamed
                        if token == active_token_str {
                            current_profile_name = Some(profile_name.clone()); // Renamed
                            break;
                        }
                    }
                    if let Some(name) = current_profile_name {
                        // Renamed
                        println!("Current active profile: {}", name); // Renamed
                    } else {
                        println!(
                            "Current active token is set, but not found under any profile name in {}.", // Renamed
                            settings.profiles_filename // Renamed
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
            let mut profiles = read_profiles(&settings)?; // Renamed
            if profiles.0.remove(&args.profile_name).is_some() {
                // Renamed
                write_profiles(&settings, &profiles)?; // Renamed
                println!(
                    "Profile '{}' deleted from {}.", // Renamed
                    args.profile_name,
                    settings.profiles_filename // Renamed
                );
            } else {
                println!(
                    "Profile '{}' not found in {}. Nothing to delete.", // Renamed
                    args.profile_name,
                    settings.profiles_filename // Renamed
                );
                anyhow::bail!("Profile not found for deletion."); // Renamed
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
                "Profiles filename [{}]: ",         // Renamed
                current_settings.profiles_filename  // Renamed
            );
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().is_empty() {
                current_settings.profiles_filename = input.trim().to_string(); // Renamed
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
