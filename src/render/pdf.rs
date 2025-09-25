// src/render/pdf.rs
use crate::domain::form::AutorisationForm;
use anyhow::{Context, Result};
use std::path::Path;
use genpdf::Element;

const TYPST_TEMPLATE: &str = include_str!("../../templates/autorisation.typ");

pub fn render_pdf(form: &AutorisationForm, school_name: Option<&str>, out: &Path) -> Result<()> {
    let typ = prepare_typst(form, school_name);

    // Try typst first (if present)
    if let Ok(typst_path) = which::which("typst") {
        let mut tmp = std::env::temp_dir();
        tmp.push("autorisation.typ");
        std::fs::write(&tmp, typ.as_bytes()).context("write typst tmp")?;
        let status = std::process::Command::new(typst_path)
            .arg("compile")
            .arg(&tmp)
            .arg("-o")
            .arg(out)
            .status()
            .context("failed to spawn typst")?;
        if status.success() {
            return Ok(());
        }
        // else fallthrough to genpdf
    }

    // Fallback: produce a presentable PDF via genpdf (requires fonts/DejaVuSans.ttf)
    fallback_pdf_genpdf(form, school_name, out)?;
    Ok(())
}

fn prepare_typst(form: &AutorisationForm, school_name: Option<&str>) -> String {
    let mut t = TYPST_TEMPLATE.to_string();
    t = t.replace("{{school_name}}", school_name.unwrap_or(""));
    t = t.replace("{{enfant_nom}}", &form.enfant.nom);
    t = t.replace(
        "{{enfant_prenom}}",
        form.enfant.prenom.clone().unwrap_or_default().as_str(),
    );
    t = t.replace("{{date}}", &form.date);
    t = t.replace("{{lieu}}", &form.lieu);
    t = t.replace("{{classe}}", form.classe.as_deref().unwrap_or(""));
    t = t.replace(
        "{{responsable_nom}}",
        form.responsable
            .as_ref()
            .map(|r| r.nom.clone())
            .unwrap_or_default()
            .as_str(),
    );
    t = t.replace(
        "{{responsable_tel}}",
        form.responsable
            .as_ref()
            .and_then(|r| r.telephone.clone())
            .unwrap_or_default()
            .as_str(),
    );
    t = t.replace("{{motif}}", form.motif.clone().unwrap_or_default().as_str());
    t
}

fn fallback_pdf_genpdf(form: &AutorisationForm, school_name: Option<&str>, out: &Path) -> Result<()> {
    use genpdf::{elements, fonts, style, Alignment, Document, PaperSize, SimplePageDecorator};

    // load fonts
    let font_family = fonts::from_files("./fonts", "DejaVuSans", None)
        .context("Failed to load font family from ./fonts (add DejaVuSans.ttf)")?;

    let mut doc = Document::new(font_family);
    doc.set_paper_size(PaperSize::A4);
    doc.set_font_size(12);

    // --- Make an owned copy of school_name so the header closure can be 'static ---
    let school_name_owned: Option<String> = school_name.map(|s| s.to_string());

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);

    // use `move` so the closure takes ownership of school_name_owned (no borrowed refs escape)
    decorator.set_header(move |page| {
        let mut layout = elements::LinearLayout::vertical();
        if page == 1 {
            if let Some(ref s) = school_name_owned {
                // push centered, bold school name
                layout.push(
                    elements::Paragraph::new(s.clone())
                        .aligned(Alignment::Center)
                        .styled(style::Style::new().with_font_size(12).bold()),
                );
                layout.push(elements::Break::new(0.5));
            }
        }
        // return the layout as an element (styled)
        layout.styled(style::Style::new().with_font_size(10))
    });

    doc.set_page_decorator(decorator);

    // content (same as before)...
    doc.push(
        elements::Paragraph::new("Autorisation de sortie")
            .aligned(Alignment::Center)
            .styled(style::Style::new().with_font_size(16).bold()),
    );
    doc.push(elements::Break::new(1.0));

    let enfant = format!(
        "Enfant : {} {}",
        form.enfant.nom,
        form.enfant.prenom.clone().unwrap_or_default()
    );
    doc.push(elements::Paragraph::new(enfant));
    doc.push(elements::Paragraph::new(format!("Date : {}", form.date)));
    doc.push(elements::Paragraph::new(format!("Lieu : {}", form.lieu)));

    if let Some(classe) = &form.classe {
        doc.push(elements::Paragraph::new(format!("Classe : {}", classe)));
    }
    if let Some(resp) = &form.responsable {
        doc.push(elements::Paragraph::new(format!("Responsable légal : {}", resp.nom)));
        if let Some(tel) = &resp.telephone {
            doc.push(elements::Paragraph::new(format!("Tél : {}", tel)));
        }
    }
    if let Some(m) = &form.motif {
        doc.push(elements::Paragraph::new(format!("Motif : {}", m)));
    }

    doc.push(elements::Break::new(2.0));
    let mut sig = elements::LinearLayout::vertical();
    sig.push(elements::Paragraph::new(
        "Fait à ___________________, le ___________________",
    ));
    sig.push(elements::Paragraph::new(
        "Signature du responsable légal : ____________________________",
    ));
    doc.push(sig);

    // write file
    doc.render_to_file(out).context("genpdf failed to render PDF")?;
    Ok(())
}

