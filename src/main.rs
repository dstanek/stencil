use clap::{CommandFactory, Parser, Subcommand};

mod config;

#[derive(Parser)]
#[command(name = "stencil")]
#[command(about = "Keeping projects in sync!")]
struct Cli {
    #[arg(
        short,
        long,
        help = "Path to the configuration file",
        default_value = ".stencil.toml"
    )]
    config: String,

    #[arg(long, help = "Disable diff output", action = clap::ArgAction::SetTrue)]
    no_diff: bool,

    #[arg(short, long = "override", help = "Override configuration value")]
    override_values: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Check,
    Sync,
}

fn main() {
    let cli = Cli::parse();

    let mut config = config::load(cli.config.as_str()).expect("failed to parse config");
    let result = config.apply_overrides(cli.override_values);
    if let Err(e) = result {
        eprintln!("Error applying overrides: {}", e);
        std::process::exit(1);
    }

    match &cli.command {
        Some(Commands::Init) => init(&config, cli.no_diff),
        Some(Commands::Check) => check(&config, cli.no_diff),
        Some(Commands::Sync) => sync(&config, cli.no_diff),
        None => Cli::command().print_long_help().unwrap(),
    }
}

fn init(config: &config::Config, no_diff: bool) {
    println!("Running check: diff={no_diff}");
    _show(config);
}

fn check(config: &config::Config, no_diff: bool) {
    println!("Running check: diff={no_diff}");
    _show(config);
}

fn sync(config: &config::Config, no_diff: bool) {
    println!("Running sync: diff={no_diff}");
    _show(config);
}

fn _show(config: &config::Config) {
    println!("Config: {:?}", config);
    println!("  Stencil:version : {:?}", config.stencil.version);
    println!("  Project:name: {:?}", config.project.name);
    println!("  Project:src: {:?}", config.project.src);
}
