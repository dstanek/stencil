// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use termcolor::{Color, ColorChoice, StandardStream};

mod diff;
mod output;
mod render;
mod target_config;

use render::RenderingIterator;
use stencil_error::StencilError;
use stencil_source::{renderables, Renderable};
use target_config::TargetConfig;

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

    #[arg(short, long = "override", help = "Override configuration value")]
    override_values: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    Plan(PlanArgs),
    Apply(ApplyArgs),
}

#[derive(Parser)]
struct InitArgs {
    #[arg(help = "Destination path")] // TODO: i hate the word dest - something better?
    dest: String,

    #[arg(help = "Stencil source (file:// or github://owner/repo)")]
    src: String,

    #[arg(long = "no-diff", help = "Disable diff output", action = clap::ArgAction::SetFalse)]
    show_diff: bool,
}

#[derive(Parser)]
struct PlanArgs {
    #[arg(help = "Destination path")]
    dest: Option<String>,
}

#[derive(Parser)]
struct ApplyArgs {
    #[arg(help = "Destination path")]
    dest: Option<String>,

    #[arg(
        short,
        long,
        help = "Automatically approve apply",
        default_value = "false"
    )]
    auto_approve: bool,

    #[arg(long = "no-diff", help = "Disable diff output", action = clap::ArgAction::SetFalse)]
    show_diff: bool,
}

fn main() {
    if let Err(err) = run() {
        // Write a colored error message to stderr
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
        output::write_bold(&mut stderr, Color::Red, format!("Error: {}\n", err)).unwrap();

        // Set a non-zero exit code
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Init(args)) => {
            let dest = PathBuf::from(&args.dest);
            init(args.show_diff, &dest, &args.src)?
        }
        Some(Commands::Plan(args)) => {
            let dest = match &args.dest {
                Some(dest) => PathBuf::from(dest),
                None => std::env::current_dir()?,
            };
            let config_path = dest.join(&cli.config);
            let mut config = TargetConfig::load(config_path.to_str().unwrap())
                .context(format!("loading config file: {}", config_path.display()))?;
            match config.apply_overrides(cli.override_values) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            plan(&config, &dest)?;
        }
        Some(Commands::Apply(args)) => {
            let dest = match &args.dest {
                Some(dest) => PathBuf::from(dest),
                None => std::env::current_dir()?,
            };
            let config_path = dest.join(&cli.config);
            let mut config = TargetConfig::load(config_path.to_str().unwrap())?;
            match config.apply_overrides(cli.override_values) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            let dest = match &args.dest {
                Some(dest) => PathBuf::from(dest),
                None => std::env::current_dir()?,
            };
            apply(&config, &dest, args.show_diff, args.auto_approve)?;
        }
        None => Cli::command().print_long_help().unwrap(),
    }

    Ok(())
}

fn init(show_diff: bool, dest: &PathBuf, src: &str) -> Result<(), StencilError> {
    println!("Initializing {}", dest.display());

    // Fail if the dest already exists
    if Path::new(dest).exists() {
        return Err(StencilError::DestinationExists(
            dest.to_string_lossy().into_owned(), // TODO: what?
        ));
    }

    // Create the destination directory
    fs::create_dir_all(dest)?;

    // TODO: ask questions from the source config

    // Create the initial config file
    let config = TargetConfig {
        stencil: target_config::ConfigStencil {
            version: "1".to_string(),
        },
        project: target_config::ConfigProject {
            name: "my_project".to_string(),
            src: src.to_string(),
        },
    };
    let mut config_path = PathBuf::from(dest);
    config_path.push(".stencil.toml");
    config.save(&config_path, &config)?;

    let iterator = create_iterator(&config)?;
    let changes = iterator
        .map(|result| result.unwrap())
        .collect::<Vec<Renderable>>();
    // Show diff and apply the changes
    if show_diff {
        diff::show_diff(&changes, &config, dest)?;
    }
    apply_changes(dest, &config)?;

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    output::write_bold(
        &mut stdout,
        Color::Green,
        format!("Successfully initialized {}", dest.display()),
    )?;
    Ok(())
}

fn plan(config: &TargetConfig, dest: &Path) -> Result<(), StencilError> {
    println!("Planning {} changes", dest.display());
    let iterator = create_iterator(config)?;
    let changes = iterator
        .map(|result| result.unwrap())
        .collect::<Vec<Renderable>>();
    _show(config);
    diff::show_diff(&changes, config, dest)?;
    Ok(())
}

fn apply(
    config: &TargetConfig,
    dest: &Path,
    show_diff: bool,
    _auto_approve: bool,
) -> Result<(), StencilError> {
    println!(
        "Applying changes from {} to {}",
        config.project.src,
        dest.display()
    );
    println!("Syncing {} from {}", dest.display(), config.project.src);
    let iterator = create_iterator(config)?;
    let changes = iterator
        .map(|result| result.unwrap())
        .collect::<Vec<Renderable>>();
    if show_diff {
        diff::show_diff(&changes, config, dest)?;
    }
    // 1. display diff
    // 2. run apply
    apply_changes(dest, config)
}

fn _show(config: &TargetConfig) {
    println!("\nConfig: {:?}", config);
    println!("  Stencil:version : {:?}", config.stencil.version);
    println!("  Project:name: {:?}", config.project.name);
    println!("  Project:src: {:?}", config.project.src);
}

// An iterator that wraps FilesystemIterator and applies the rendering logic

fn apply_changes(dest: &Path, config: &TargetConfig) -> Result<(), StencilError> {
    let iterator = create_iterator(config)?;
    for entry in iterator {
        match entry {
            Ok(Renderable::Directory(dir)) => {
                let path = dest.join(&dir.relative_path);
                // println!("Creating directory: {:?}", path);
                fs::create_dir_all(path)?;
                // println!("Successfully created directory: {:?}", path);
            }
            Ok(Renderable::File(file)) => {
                let path = dest.join(&file.relative_path);
                // println!("Creating file: {:?}", path);
                fs::write(path, file.content)?;
                //  println!("Successfully created file: {:?}", path);

                //println!("File: {:?} {:?}", file.path, file.content);
                //let empty = File::empty();
                //diff::show_diff(
                //    &Renderable::File(empty),
                //    &Renderable::File(file),
                //)?;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// Create an iterator that wraps FilesystemIterator and filters out files that are in the ignore list
//pub struct CheckIterator<I: Iterator<Item = Renderable>> {
//    iterator: I,
//    ignore: Vec<String>,
//}

//impl Iterator for CheckIterator {
//    type Item = Result<Renderable, StencilError>;
//
//    fn next(&mut self) -> Option<Self::Item> {
//    while let Some(renderable) = self.iterator.next() {
//       match renderable {
//          Ok(Renderable::File(file)) => {
//   // continue if the file is in a list of files to ignore and the file is already in the destination
//    if self
//         .ignore
//          .contains(&file.relative_path.to_str().unwrap().to_string())
//       {
//            continue;
//         }
//          return Some(Ok(Renderable::File(file)));
//       }
//        Ok(Renderable::Directory(dir)) => {
//             return Some(Ok(Renderable::Directory(dir)));
//          }
//           Err(e) => return Some(Err(e)),
//        }
//     }
//      None
//   }
//}

//impl CheckIterator<I: Iterator<Item = Renderable>> {
//   pub fn new(iterator: FilesystemIterator, ignore: Vec<String>) -> CheckIterator {
//      CheckIterator { iterator, ignore }
// }
//}

pub fn create_iterator(config: &TargetConfig) -> Result<RenderingIterator, StencilError> {
    let iterator = renderables(&config.project.src)?;
    //let iterator = match FilesystemCrawler::new(stencil_path.as_path()).crawl() {
    //   Ok(iterator) => iterator,
    //  Err(e) => {
    //     eprintln!("Error: {}", e);
    //    std::process::exit(1);
    //}
    //};
    let iterator = RenderingIterator::new(iterator, config);
    Ok(iterator)
    //let mut ignore = Vec::new();
    // ignore.push(".gitignore".to_string());
    //ignore.push("README.md".to_string());
    // ignore.push("main.tf".to_string());
    // ignore.push("outputs.tf".to_string());
    // ignore.push("terraform.tf".to_string());
    // ignore.push("variables.tf".to_string());
    //return Ok(CheckIterator::new(iterator, ignore));
}

// TODO: implement a way to ignore certain files
//fn filter_files(iterator: FilesystemIterator, ignore: Vec<String>) -> Vec<Renderable> {
//    let d = iterator
//        .filter_map(|result| match result {
//            Ok(Renderable::File(file)) => {
//                if !ignore.contains(&file.relative_path.to_str().unwrap().to_string()) {
//                    Some(Renderable::File(file))
//                } else {
//                    None
//                }
//            }
//            Ok(Renderable::Directory(dir)) => Some(Renderable::Directory(dir)),
//            Err(_) => None,
//        })
//        .collect();
//    d
//}
