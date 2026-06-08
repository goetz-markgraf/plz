mod cli;
mod config;
mod detection;
mod api;
mod models;
mod output;

use cli::PlzArgs;
use config::Config;
use api::{PlzClient, PlzError};
use clap::{Parser, CommandFactory};

#[tokio::main]
async fn main() {
    let args = PlzArgs::parse();

    if args.query.is_empty() {
        eprintln!("Error: Query cannot be empty.");
        eprintln!();
        eprint!("{}", PlzArgs::command().render_help());
        std::process::exit(1);
    }

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let model = if let Some(ref m) = args.model {
        m.clone()
    } else if let Some(ref m) = config.model {
        m.clone()
    } else {
        // No model configured — list models and exit
        match models::list_models(&config.endpoint, &config.api_key).await {
            Ok(models_list) => {
                models::format_models(&models_list);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error listing models: {}", e);
                eprintln!("Hint: You can specify a model with --model <name>");
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = config.validate_for_query() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let system_prompt = detection::build_system_prompt();
    let user_prompt = detection::build_user_prompt(&args.query);

    let client = PlzClient::new(config.endpoint, config.api_key);

    let content = match client.chat_completion(&model, &system_prompt, &user_prompt, config.max_tokens).await {
        Ok(c) => c,
        Err(PlzError::InvalidApiKey(_msg)) => {
            eprintln!("{}", PlzError::InvalidApiKey("".to_string()));
            std::process::exit(1);
        }
        Err(PlzError::HttpError(status, msg)) => {
            eprintln!("{}", PlzError::HttpError(status, msg));
            std::process::exit(1);
        }
        Err(PlzError::NetworkError(msg)) => {
            eprintln!("{}", PlzError::NetworkError(msg));
            std::process::exit(1);
        }
        Err(PlzError::Anyhow(e)) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    if args.command_only {
        output::format_command_only(&content);
    } else {
        output::format_response(&content);
    }
}
