use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "mbsite",
    version,
    about = "Build and deploy Markdown-first static sites"
)]
pub struct Cli {
    /// Path to the site configuration.
    #[arg(short, long, default_value = "mambo.toml", global = true)]
    pub config: PathBuf,

    /// Diagnostic output format.
    #[arg(long, value_enum, default_value_t = DiagnosticFormat::Text, global = true)]
    pub diagnostics: DiagnosticFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum DiagnosticFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Parse and validate the complete site without writing generated files.
    Check,
    /// Compile content and build the configured static React renderer.
    Build {
        /// Stop after generating content and assets; do not run the renderer.
        #[arg(long)]
        content_only: bool,
    },
    /// Bootstrap an empty repository with the default `MamboSite` scaffold.
    Init {
        /// Empty directory to initialize. Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Refresh paths owned by an existing `MamboSite` scaffold.
        #[arg(long)]
        force: bool,
    },
    /// Build, push committed work, and trigger GitHub Pages when needed.
    Deploy {
        /// Run local validation and print the remote action without changing Git or GitHub.
        #[arg(long)]
        dry_run: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_check_with_default_config() {
        let cli = Cli::try_parse_from(["mbsite", "check"]).unwrap();
        assert_eq!(cli.config, PathBuf::from("mambo.toml"));
        assert_eq!(cli.diagnostics, DiagnosticFormat::Text);
        assert!(matches!(cli.command, Command::Check));
    }

    #[test]
    fn parses_lifecycle_flags() {
        let cli = Cli::try_parse_from([
            "mbsite",
            "build",
            "--content-only",
            "--config",
            "sites/wiki.toml",
            "--diagnostics",
            "json",
        ])
        .unwrap();
        assert_eq!(cli.config, PathBuf::from("sites/wiki.toml"));
        assert_eq!(cli.diagnostics, DiagnosticFormat::Json);
        assert!(matches!(cli.command, Command::Build { content_only: true }));

        let cli = Cli::try_parse_from(["mbsite", "init", "new-site", "--force"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Init { path, force: true } if path == std::path::Path::new("new-site")
        ));

        let cli = Cli::try_parse_from(["mbsite", "deploy", "--dry-run"]).unwrap();
        assert!(matches!(cli.command, Command::Deploy { dry_run: true }));
    }
}
