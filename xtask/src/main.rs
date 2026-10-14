use clap::{Parser, Subcommand};
mod commands;

#[derive(Parser)]
#[command(about = "Build and preview the iso-types documentation and examples")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Build {
        #[command(subcommand)]
        target: BuildCommand,
    },
    Preview {
        #[command(subcommand)]
        target: PreviewCommand,
    },
}

#[derive(Subcommand)]
enum BuildCommand {
    Book,
    LlmsTxt,
    Web,
}

#[derive(Subcommand)]
enum PreviewCommand {
    Web,
}

fn main() -> anyhow::Result<()> {
    let root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    match Cli::parse().command {
        Command::Build { target } => match target {
            BuildCommand::Book => stayhydated_xtask::book::build_workspace_book(&root),
            BuildCommand::LlmsTxt => stayhydated_xtask::llms::build_workspace_llms(
                &root,
                "https://stayhydated.github.io/iso-types",
                None,
            ),
            BuildCommand::Web => commands::build_web::run(),
        },
        Command::Preview {
            target: PreviewCommand::Web,
        } => commands::preview_web::run(),
    }
}
