use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use mambosite_core::{CompileOutcome, Config, Diagnostic, Severity};

#[derive(Debug, Parser)]
#[command(
    name = "mambosite",
    version,
    about = "Compile Markdown sites into typed TypeScript content"
)]
struct Cli {
    /// Path to the site configuration.
    #[arg(short, long, default_value = "mambo.toml", global = true)]
    config: PathBuf,

    /// Diagnostic output format.
    #[arg(long, value_enum, default_value_t = DiagnosticFormat::Text, global = true)]
    diagnostics: DiagnosticFormat,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum DiagnosticFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and validate the complete site without writing generated files.
    Check,
    /// Parse, validate, and generate TypeScript content modules.
    Build,
}

fn main() -> ExitCode {
    run(&Cli::parse())
}

fn run(cli: &Cli) -> ExitCode {
    let config = match Config::load(&cli.config) {
        Ok(config) => config,
        Err(diagnostics) => {
            if let Err(error) = print_diagnostics(&diagnostics, cli.diagnostics) {
                eprintln!("error: could not print diagnostics: {error}");
                return ExitCode::from(2);
            }
            return ExitCode::FAILURE;
        }
    };
    let output_dir = config.typescript_out.clone();
    let CompileOutcome { site, diagnostics } = mambosite_core::Compiler::new(config).compile();

    if let Err(error) = print_diagnostics(&diagnostics, cli.diagnostics) {
        eprintln!("error: could not print diagnostics: {error}");
        return ExitCode::from(2);
    }
    if diagnostics.iter().any(Diagnostic::is_error) {
        return ExitCode::FAILURE;
    }

    let Some(site) = site else {
        eprintln!("error: compilation produced neither a site nor an error diagnostic");
        return ExitCode::from(2);
    };
    let page_count = site.pages.len();

    match &cli.command {
        Command::Check => {
            print_success(
                cli.diagnostics,
                &format!("checked {page_count} page(s) successfully"),
            );
        }
        Command::Build => {
            if let Err(error) = mambosite_codegen_ts::generate_to(&site, &output_dir) {
                eprintln!("error: TypeScript generation failed: {error}");
                return ExitCode::FAILURE;
            }
            print_success(
                cli.diagnostics,
                &format!("generated {page_count} page(s) in {}", output_dir.display()),
            );
        }
    }

    ExitCode::SUCCESS
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_check_with_default_config() {
        let cli = Cli::try_parse_from(["mambosite", "check"]).unwrap();
        assert_eq!(cli.config, PathBuf::from("mambo.toml"));
        assert_eq!(cli.diagnostics, DiagnosticFormat::Text);
        assert!(matches!(cli.command, Command::Check));
    }

    #[test]
    fn accepts_global_options_after_subcommand() {
        let cli = Cli::try_parse_from([
            "mambosite",
            "build",
            "--config",
            "sites/wiki.toml",
            "--diagnostics",
            "json",
        ])
        .unwrap();
        assert_eq!(cli.config, PathBuf::from("sites/wiki.toml"));
        assert_eq!(cli.diagnostics, DiagnosticFormat::Json);
        assert!(matches!(cli.command, Command::Build));
    }
}
