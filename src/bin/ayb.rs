use ayb::ayb_db::models::{DBType, EntityType};
use ayb::client::config::ClientConfig;
use ayb::client::http::AybClient;
use ayb::formatting::TabularFormatter;
use ayb::http::structs::{EntityDatabasePath, ProfileLinkUpdate};
use ayb::server::config::{config_to_toml, default_server_config};
use ayb::server::server_runner::run_server;
use clap::builder::ValueParser;
use clap::{arg, command, value_parser, Command, ValueEnum};
use directories::ProjectDirs;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

fn entity_database_parser(value: &str) -> Result<EntityDatabasePath, String> {
    let re = Regex::new(r"^(\S+)/(\S+)$").unwrap();
    if re.is_match(value) {
        let captures = re.captures(value).unwrap();
        Ok(EntityDatabasePath {
            entity: captures.get(1).map_or("", |m| m.as_str()).to_string(),
            database: captures.get(2).map_or("", |m| m.as_str()).to_string(),
        })
    } else {
        Err("Argument must be formatted as 'entity/database'".to_string())
    }
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table = 0,
    Csv = 1,
}

impl OutputFormat {
    pub fn to_str(&self) -> &str {
        match self {
            OutputFormat::Table => "table",
            OutputFormat::Csv => "csv",
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let matches = command!()
        .subcommand(
            Command::new("server")
                .about("Run an HTTP server")
                .arg(arg!(--config <FILE> "Path to the server's configuration file")
                     .value_parser(value_parser!(PathBuf))
                     .env("AYB_SERVER_CONFIG_FILE")
                     .default_value("./ayb.toml"))
        )
        .subcommand(
            Command::new("default_server_config")
                .about("Print a default configuration file for a server")
        )
        .subcommand(
            Command::new("client")
                .about("Connect to an HTTP server")
                .arg(
                    arg!(--config <FILE> "Path to the client's configuration file")
                        .value_parser(value_parser!(PathBuf))
                        .env("AYB_CLIENT_CONFIG_FILE")
                )
                .arg(
                    arg!(--url <VALUE> "The server URL")
                        .env("AYB_SERVER_URL")
                        .required(false)
                )
                .arg(
                    arg!(--token <VALUE> "A client API token")
                        .env("AYB_API_TOKEN")
                        .required(false)
                )
                .subcommand(
                    Command::new("create_database")
                        .about("Create a database")
                        .arg(arg!(<database> "The database to create (e.g., entity/database.sqlite")
                             .value_parser(ValueParser::new(entity_database_parser))
                             .required(true)
                        )
                        .arg(
                            arg!(<type> "The type of DB")
                                .value_parser(value_parser!(DBType))
                                .default_value(DBType::Sqlite.to_str())
                                .required(false)
                        ),
                )
                .subcommand(
                    Command::new("query")
                        .about("Query a database")
                        .arg(arg!(<database> "The database to which to connect (e.g., entity/database.sqlite")
                             .value_parser(ValueParser::new(entity_database_parser))
                             .required(true)
                        )
                        .arg(arg!(<query> "The query to execute")
                             .required(true))
                        .arg(
                            arg!(--format <type> "The format in which to output the result")
                                .value_parser(value_parser!(OutputFormat))
                                .default_value(OutputFormat::Table.to_str())
                                .required(false)),
                )
                .subcommand(
                    Command::new("register")
                        .about("Register a user/organization")
                        .arg(arg!(<entity> "The entity to create")
                             .required(true))
                        .arg(arg!(<email_address> "The email address of the entity")
                             .required(true))
                        .arg(
                            arg!(<type> "The type of entity")
                                .value_parser(value_parser!(EntityType))
                                .default_value(EntityType::User.to_str())
                                .required(false)),
                )
                .subcommand(
                    Command::new("confirm")
                        .about("Confirm an email-based login/registration")
                        .arg(arg!(<authentication_token> "The authentication token")
                             .required(true))
                )
                .subcommand(
                    Command::new("log_in")
                        .about("Log in to ayb via email authentication")
                        .arg(arg!(<entity> "The entity to log in as")
                             .required(true))
                )
                .subcommand(
                    Command::new("list")
                        .about("List the databases of a given entity")
                        .arg(arg!(<entity> "The entity to query")
                            .required(true))
                        .arg(
                            arg!(--format <type> "The format in which to output the result")
                                .value_parser(value_parser!(OutputFormat))
                                .default_value(OutputFormat::Table.to_str())
                                .required(false)),
                )
                .subcommand(
                    Command::new("profile")
                        .about("Show the profile of an entity")
                        .arg(arg!(<entity> "The entity to query")
                            .required(true))
                        .arg(
                            arg!(--format <type> "The format in which to output the result")
                                .value_parser(value_parser!(OutputFormat))
                                .default_value(OutputFormat::Table.to_str())
                                .required(false))
                )
                .subcommand(
                    Command::new("update_profile")
                        .about("Update the profile of an entity")
                        .arg(arg!(<entity> "The entity to update").required(true))
                        .arg(arg!(--display_name <value> "New display name").required(false))
                        .arg(arg!(--description <value> "New description").required(false))
                        .arg(arg!(--organization <value> "New organization").required(false))
                        .arg(arg!(--location <value> "New location").required(false))
                        .arg(
                            arg!(--links <value> "New links")
                                .required(false)
                                .num_args(0..)
                        )
                )
                .subcommand(
                    Command::new("set_default_url")
                        .about("Set the default server URL for future requests in ayb.json")
                        .arg(arg!(<url> "The URL to use in the future")
                             .required(true))
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        if let Some(config) = matches.get_one::<PathBuf>("config") {
            run_server(config).await?;
        }
    } else if let Some(_matches) = matches.subcommand_matches("default_server_config") {
        match config_to_toml(default_server_config()) {
            Ok(config) => println!("{}", config),
            Err(err) => println!("Error: {}", err),
        }
    } else if let Some(matches) = matches.subcommand_matches("client") {
        let config_path = if let Some(path) = matches.get_one::<PathBuf>("config") {
            path.clone()
        } else {
            ProjectDirs::from("org", "ayb", "ayb")
                .expect("can't determine ayb project directory directory")
                .config_dir()
                .join("ayb.json")
        };
        let mut config = ClientConfig::from_file(&config_path)?;

        if let Some(matches) = matches.subcommand_matches("set_default_url") {
            if let Some(url) = matches.get_one::<String>("url") {
                config.default_url = Some(url.to_string());
                config.to_file(&config_path)?;
                println!("Saved {} as new default_url", url);
                return Ok(());
            }
        }

        let url = if let Some(server_url) = matches.get_one::<String>("url") {
            if config.default_url.is_none() {
                config.default_url = Some(server_url.to_string());
                config.to_file(&config_path)?;
            }
            server_url.to_string()
        } else if let Some(ref server_url) = config.default_url {
            server_url.to_string()
        } else {
            panic!("Server URL is required through --url parameter, AYB_SERVER_URL environment variable, or default_url in {}", config_path.display());
        };

        let token = matches
            .get_one::<String>("token")
            .or(config.authentication.get(&url))
            .cloned();
        let client = AybClient {
            base_url: url.to_string(),
            api_token: token,
        };
        if let Some(matches) = matches.subcommand_matches("create_database") {
            if let (Some(entity_database), Some(db_type)) = (
                matches.get_one::<EntityDatabasePath>("database"),
                matches.get_one::<DBType>("type"),
            ) {
                match client
                    .create_database(&entity_database.entity, &entity_database.database, db_type)
                    .await
                {
                    Ok(_response) => {
                        println!(
                            "Successfully created {}/{}",
                            entity_database.entity, entity_database.database
                        );
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("register") {
            if let (Some(entity), Some(email_address), Some(entity_type)) = (
                matches.get_one::<String>("entity"),
                matches.get_one::<String>("email_address"),
                matches.get_one::<EntityType>("type"),
            ) {
                match client.register(entity, email_address, entity_type).await {
                    Ok(_response) => {
                        println!("Check your email to finish registering {}", entity);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("confirm") {
            if let Some(authentication_token) = matches.get_one::<String>("authentication_token") {
                match client.confirm(authentication_token).await {
                    Ok(api_token) => {
                        config
                            .authentication
                            .insert(url.clone(), api_token.token.clone());
                        config.to_file(&config_path)?;
                        println!(
                            "Successfully authenticated {} and saved token {}",
                            api_token.entity, api_token.token
                        );
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("log_in") {
            if let Some(entity) = matches.get_one::<String>("entity") {
                match client.log_in(entity).await {
                    Ok(_response) => {
                        println!("Check your email to finish logging in {}", entity);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("profile") {
            if let (Some(entity), Some(format)) = (
                matches.get_one::<String>("entity"),
                matches.get_one::<OutputFormat>("format"),
            ) {
                match client.entity_details(entity).await {
                    Ok(response) => match format {
                        OutputFormat::Table => response.profile.generate_table()?,
                        OutputFormat::Csv => response.profile.generate_csv()?,
                    },
                    Err(err) => println!("Error: {}", err),
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("update_profile") {
            if let Some(entity) = matches.get_one::<String>("entity") {
                let mut profile_update = HashMap::new();
                if let Some(display_name) = matches.get_one::<String>("display_name").cloned() {
                    profile_update.insert("display_name".to_owned(), Some(display_name));
                }

                if let Some(description) = matches.get_one::<String>("description").cloned() {
                    profile_update.insert("description".to_owned(), Some(description));
                }

                if let Some(organization) = matches.get_one::<String>("organization").cloned() {
                    profile_update.insert("organization".to_owned(), Some(organization));
                }

                if let Some(location) = matches.get_one::<String>("location").cloned() {
                    profile_update.insert("location".to_owned(), Some(location));
                }

                if matches.get_many::<String>("links").is_some() {
                    profile_update.insert(
                        "links".to_owned(),
                        Some(serde_json::to_string(
                            &matches
                                .get_many::<String>("links")
                                .map(|v| v.into_iter().collect::<Vec<&String>>())
                                .map(|v| {
                                    v.into_iter()
                                        .map(|v| ProfileLinkUpdate { url: v.clone() })
                                        .collect::<Vec<ProfileLinkUpdate>>()
                                }),
                        )?),
                    );
                }

                match client.update_profile(entity, &profile_update).await {
                    Ok(_) => println!("Successfully updated profile"),
                    Err(err) => println!("Error: {}", err),
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("list") {
            if let (Some(entity), Some(format)) = (
                matches.get_one::<String>("entity"),
                matches.get_one::<OutputFormat>("format"),
            ) {
                match client.entity_details(entity).await {
                    Ok(response) => {
                        if response.databases.is_empty() {
                            println!("No queryable databases owned by {}", entity);
                        } else {
                            match format {
                                OutputFormat::Table => response.databases.generate_table()?,
                                OutputFormat::Csv => response.databases.generate_csv()?,
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        } else if let Some(matches) = matches.subcommand_matches("query") {
            if let (Some(entity_database), Some(query), Some(format)) = (
                matches.get_one::<EntityDatabasePath>("database"),
                matches.get_one::<String>("query"),
                matches.get_one::<OutputFormat>("format"),
            ) {
                match client
                    .query(&entity_database.entity, &entity_database.database, query)
                    .await
                {
                    Ok(query_result) => {
                        if !query_result.rows.is_empty() {
                            match format {
                                OutputFormat::Table => query_result.generate_table()?,
                                OutputFormat::Csv => query_result.generate_csv()?,
                            }
                        }
                        println!("\nRows: {}", query_result.rows.len());
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        }
    }

    Ok(())
}
