// src/render/pdf.rs
#![forbid(unsafe_code)]

use crate::domain::form::AutorisationForm;
use crate::domain::format::human_date_fr;
use anyhow::{Context, Result};
use genpdf::Element;
use genpdf::{elements, fonts, style, Alignment, Document, PaperSize, SimplePageDecorator};
use std::path::Path;

pub fn render_pdf(form: &AutorisationForm, school_name: Option<&str>, out: &Path) -> Result<()> {
    // chemins candidats pour trouver DejaVuSans.ttf
    let candidates = [
        "./fonts",
        "./assets/fonts",
        "/usr/share/fonts/truetype/dejavu",
        "/usr/share/fonts",
        "/usr/local/share/fonts",
    ];

    let mut font_family_opt = None;
    for dir in &candidates {
        if std::path::Path::new(dir).exists() {
            if let Ok(f) = fonts::from_files(dir, "DejaVuSans", None) {
                font_family_opt = Some(f);
                break;
            }
        }
    }

    let font_family = font_family_opt
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Impossible de charger une font pour genpdf. Placez DejaVuSans.ttf dans ./fonts ou installez fonts-dejavu."
            )
        })?;

    // Document
    let mut doc = Document::new(font_family);
    doc.set_paper_size(PaperSize::A4);
    doc.set_font_size(11);

    // Header (simple). Pas de footer compliqué pour compatibilité genpdf 0.2.0
    let school_name_owned = school_name.map(|s| s.to_string());
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(18); // points
    decorator.set_header(move |page| {
        let mut layout = elements::LinearLayout::vertical();
        if page == 1 {
            if let Some(ref s) = school_name_owned {
                if !s.is_empty() {
                    layout.push(
                        elements::Paragraph::new(s.clone())
                            .aligned(Alignment::Center)
                            .styled(style::Style::new().with_font_size(14).bold()),
                    );
                    layout.push(elements::Break::new(0.3));
                }
            }
        }
        layout.styled(style::Style::new().with_font_size(9))
    });
    doc.set_page_decorator(decorator);

    // Title
    doc.push(
        elements::Paragraph::new("Autorisation de sortie")
            .aligned(Alignment::Center)
            .styled(style::Style::new().with_font_size(18).bold()),
    );
    doc.push(elements::Break::new(0.8));

    // Contenu principal (vertical)
    let mut content = elements::LinearLayout::vertical();

    content.push(
        elements::Paragraph::new(format!(
            "Enfant : {} {}",
            form.enfant.nom,
            form.enfant.prenom.clone().unwrap_or_default()
        ))
        .styled(style::Style::new().with_font_size(12).bold()),
    );
    content.push(elements::Break::new(0.2));

    let date_str = human_date_fr(&form.date);
    content.push(elements::Paragraph::new(format!("Date : {date_str}")));
    content.push(elements::Paragraph::new(format!("Lieu : {}", form.lieu)));

    if let Some(classe) = &form.classe {
        content.push(elements::Paragraph::new(format!("Classe : {classe}")));
    }

    if let Some(motif) = &form.motif {
        content.push(elements::Break::new(0.3));
        content.push(elements::Paragraph::new("Motif :").styled(style::Style::new().bold()));
        content.push(elements::Paragraph::new(motif.clone()));
    }

    content.push(elements::Break::new(1.0));

    if let Some(resp) = &form.responsable {
        content.push(
            elements::Paragraph::new(format!("Responsable légal : {}", resp.nom))
                .styled(style::Style::new().with_font_size(11)),
        );
        if let Some(tel) = &resp.telephone {
            content.push(elements::Paragraph::new(format!("Tél : {tel}")));
        }
        content.push(elements::Break::new(1.0));
    }

    // Signature block (stacked — droite alignée pour la ligne de signature)
    content.push(elements::Paragraph::new(
        "Fait à _______________________, le _______________________",
    ));
    content.push(elements::Break::new(1.0));
    content.push(elements::Paragraph::new("Signature du responsable légal :"));
    content.push(elements::Break::new(1.0));
    content.push(
        elements::Paragraph::new("____________________________")
            .aligned(Alignment::Right)
            .styled(style::Style::new().with_font_size(11)),
    );

    doc.push(content);

    // Render
    doc.render_to_file(out)
        .context("échec lors du rendu PDF avec genpdf")?;

    Ok(())
}
