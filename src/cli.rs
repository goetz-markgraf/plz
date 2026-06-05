use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "plz",
    about = "Convert natural language into shell commands",
    version,
    disable_version_flag = true
)]
pub struct PlzArgs {
    /// Natural language query
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Override the model from config
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Output only the raw command
    #[arg(long = "command-only")]
    pub command_only: bool,

    /// Print version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
