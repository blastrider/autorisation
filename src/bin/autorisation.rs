// src/bin/autorisation.rs
#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use autorisation::domain::form::AutorisationForm;
use autorisation::infra::fs::resolve_out_path;
use autorisation::render::{markdown, pdf};

#[derive(Parser)]
#[command(
    name = "autorisation",
    about = "Génère une autorisation de sortie (PDF/MD) - offline"
)]
struct Cli {
    #[arg(long)]
    input: Option<String>,

    #[arg(long, conflicts_with = "input")]
    interactive: bool,

    #[arg(long, default_value = "autorisation_sortie.pdf")]
    out: String,

    #[arg(long)]
    md: Option<String>,

    #[arg(long)]
    school_name: Option<String>,
}

fn main() -> Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    info!("Start autorisation CLI");

    let form = if cli.interactive {
        AutorisationForm::from_interactive()?
    } else if let Some(path) = cli.input {
        AutorisationForm::from_file(&path)
            .with_context(|| format!("failed to load input file '{path}'"))?
    } else {
        anyhow::bail!("Either --input <file> or --interactive must be provided");
    };

    form.validate().context("validation failed")?;

    if let Some(md_out) = cli.md.as_ref() {
        let md = markdown::render_markdown(&form, cli.school_name.as_deref())?;
        autorisation::infra::fs::write_atomic(md_out, md.as_bytes())
            .context("failed to write markdown output")?;
        info!("Wrote markdown {}", md_out);
    }

    let out_path = resolve_out_path(&cli.out)?;
    if let Err(e) = pdf::render_pdf(&form, cli.school_name.as_deref(), &out_path) {
        error!("PDF generation failed: {:?}", e);
        return Err(e).context("PDF generation failed");
    }
    info!("Wrote PDF {}", out_path.display());
    Ok(())
}
