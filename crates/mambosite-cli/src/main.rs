mod cli;
mod commands;
mod process;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Command, DiagnosticFormat};
use commands::CommandError;
use commands::build::{BuildMode, BuildReport};
use commands::deploy::{DeployAction, DeployReport};
use mambosite_core::{Config, Diagnostic, Severity};
use process::ChildStdout;

fn main() -> ExitCode {
    run(&Cli::parse())
}

fn run(cli: &Cli) -> ExitCode {
    match execute(cli) {
        Ok(outcome) => {
            if let Some(diagnostics) = outcome.diagnostics() {
                if let Err(error) = print_diagnostics(diagnostics, cli.diagnostics) {
                    eprintln!("error: could not print diagnostics: {error}");
                    return ExitCode::from(2);
                }
            }
            print_success(cli.diagnostics, &outcome.message());
            ExitCode::SUCCESS
        }
        Err(CommandError::Diagnostics(diagnostics)) => {
            if let Err(error) = print_diagnostics(&diagnostics, cli.diagnostics) {
                eprintln!("error: could not print diagnostics: {error}");
                return ExitCode::from(2);
            }
            ExitCode::FAILURE
        }
        Err(CommandError::Message(message)) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

enum Outcome {
    Build(BuildMode, BuildReport),
    Init(commands::init::InitReport),
    Deploy(DeployReport),
}

impl Outcome {
    fn diagnostics(&self) -> Option<&[Diagnostic]> {
        match self {
            Self::Build(_, report) => Some(&report.diagnostics),
            Self::Deploy(report) => Some(&report.build.diagnostics),
            Self::Init(_) => None,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Build(BuildMode::Check, report) => {
                format!("checked {} page(s) successfully", report.page_count)
            }
            Self::Build(BuildMode::ContentOnly, report) => {
                let generated = report
                    .generated_dir
                    .as_deref()
                    .expect("content build has generated output");
                let assets = report
                    .assets_dir
                    .as_deref()
                    .expect("content build has generated assets");
                format!(
                    "generated {} page(s) in {} and assets in {}",
                    report.page_count,
                    generated.display(),
                    assets.display()
                )
            }
            Self::Build(BuildMode::Full, report) => report.artifact_dir.as_ref().map_or_else(
                || {
                    format!(
                        "generated {} page(s); renderer is disabled",
                        report.page_count
                    )
                },
                |artifact| {
                    format!(
                        "built {} page(s) and static output in {}",
                        report.page_count,
                        artifact.display()
                    )
                },
            ),
            Self::Init(report) => format!(
                "initialized {} scaffold file(s) in {}",
                report.file_count,
                report.target.display()
            ),
            Self::Deploy(report) => {
                let prefix = if report.dry_run {
                    "dry run complete; would"
                } else {
                    "deployment started;"
                };
                match report.action {
                    DeployAction::Push { commits } => {
                        format!("{prefix} push {commits} committed change(s)")
                    }
                    DeployAction::Dispatch => {
                        format!("{prefix} dispatch the Pages workflow for the current commit")
                    }
                }
            }
        }
    }
}

fn execute(cli: &Cli) -> Result<Outcome, CommandError> {
    let child_stdout = match cli.diagnostics {
        DiagnosticFormat::Text => ChildStdout::Stdout,
        DiagnosticFormat::Json => ChildStdout::Stderr,
    };
    match &cli.command {
        Command::Init { path, force } => commands::init::run(path, *force).map(Outcome::Init),
        Command::Check => commands::build::run(load_config(cli)?, BuildMode::Check, child_stdout)
            .map(|report| Outcome::Build(BuildMode::Check, report)),
        Command::Build { content_only } => {
            let mode = if *content_only {
                BuildMode::ContentOnly
            } else {
                BuildMode::Full
            };
            commands::build::run(load_config(cli)?, mode, child_stdout)
                .map(|report| Outcome::Build(mode, report))
        }
        Command::Deploy { dry_run } => {
            commands::deploy::run(&load_config(cli)?, *dry_run, child_stdout).map(Outcome::Deploy)
        }
    }
}

fn load_config(cli: &Cli) -> Result<Config, CommandError> {
    Config::load(&cli.config).map_err(CommandError::Diagnostics)
}

fn print_diagnostics(
    diagnostics: &[Diagnostic],
    format: DiagnosticFormat,
) -> Result<(), serde_json::Error> {
    match format {
        DiagnosticFormat::Text => {
            for diagnostic in diagnostics {
                print_text_diagnostic(diagnostic);
            }
            Ok(())
        }
        DiagnosticFormat::Json => {
            serde_json::to_writer_pretty(std::io::stdout().lock(), diagnostics)?;
            println!();
            Ok(())
        }
    }
}

fn print_text_diagnostic(diagnostic: &Diagnostic) {
    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    };
    if let Some(location) = &diagnostic.primary {
        eprintln!(
            "{}:{}:{}: {severity}[{}]: {}",
            location.path,
            location.span.start.line,
            location.span.start.column,
            diagnostic.code,
            diagnostic.message
        );
    } else {
        eprintln!("{severity}[{}]: {}", diagnostic.code, diagnostic.message);
    }
    if let Some(help) = &diagnostic.help {
        eprintln!("  help: {help}");
    }
    for related in &diagnostic.related {
        eprintln!(
            "  related: {}:{}:{}",
            related.path, related.span.start.line, related.span.start.column
        );
    }
    for note in &diagnostic.notes {
        eprintln!("  note: {note}");
    }
}

fn print_success(format: DiagnosticFormat, message: &str) {
    match format {
        DiagnosticFormat::Text => println!("{message}"),
        // Preserve stdout as a single machine-readable JSON value.
        DiagnosticFormat::Json => eprintln!("{message}"),
    }
}
