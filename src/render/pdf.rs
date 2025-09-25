// src/render/pdf.rs
#![forbid(unsafe_code)]

use crate::domain::form::AutorisationForm;
use crate::domain::format::human_date_fr;
use anyhow::{Context, Result};
use genpdf::{
    elements, fonts, style, Alignment, Document, Element, Margins, PaperSize, SimplePageDecorator,
};
use std::env;
use std::path::Path;

/// Page margins (defaults) — gauche, droite, haut, bas (modifiable)
const MARGIN_LEFT_MM: f64 = 20.0;
const MARGIN_RIGHT_MM: f64 = 20.0;
const MARGIN_TOP_MM: f64 = 18.0;
const MARGIN_BOTTOM_MM: f64 = 18.0;

/// Typography (points)
const BASELINE_PT: u8 = 8; // baseline grid (used for spacing math)
const H1_PT: u8 = 32;
const H2_PT: u8 = 24;
const H3_PT: u8 = 18;
const BODY_PT: u8 = 16;
const CAPTION_PT: u8 = 12;

/// Grid
const GRID_COLS: usize = 12;

pub fn render_pdf(form: &AutorisationForm, school_name: Option<&str>, out: &Path) -> Result<()> {
    // page size env override (A4|LETTER)
    let page_size_env = env::var("AUT_PAGE_SIZE").unwrap_or_else(|_| "A4".into());
    let paper = match page_size_env.to_uppercase().as_str() {
        "LETTER" | "USLETTER" | "US_LETTER" => PaperSize::Letter,
        _ => PaperSize::A4,
    };

    // Fonts: try primary (env) then fallback to DejaVuSans in ./fonts
    let font_name_env = env::var("AUT_FONT_FAMILY").ok();
    let font_candidates = [
        font_name_env.as_deref().unwrap_or("Inter"),
        "DejaVuSans",
        "LiberationSans",
    ];

    let font_dirs = [
        "./fonts",
        "./assets/fonts",
        "/usr/share/fonts/truetype/dejavu",
        "/usr/share/fonts",
        "/usr/local/share/fonts",
    ];

    let mut font_family_opt = None;
    'outer: for dir in &font_dirs {
        if std::path::Path::new(dir).exists() {
            for fname in &font_candidates {
                if let Ok(ff) = fonts::from_files(dir, fname, None) {
                    font_family_opt = Some(ff);
                    break 'outer;
                }
            }
        }
    }

    let font_family = font_family_opt.ok_or_else(|| {
        anyhow::anyhow!(
            "Impossible de charger une font pour genpdf. Place DejaVuSans.ttf or Inter.ttf in ./fonts."
        )
    })?;

    // Document
    let mut doc = Document::new(font_family);
    doc.set_paper_size(paper);
    doc.set_title("Autorisation de sortie");

    // Base font size (document default) = BODY_PT
    doc.set_font_size(BODY_PT);

    // Page decorator (uniform margins via SimplePageDecorator)
    let mut decorator = SimplePageDecorator::new();
    // Use the minimum margin as SimplePageDecorator::set_margins expects a single value in points
    let min_margin_mm = MARGIN_LEFT_MM
        .min(MARGIN_RIGHT_MM)
        .min(MARGIN_TOP_MM)
        .min(MARGIN_BOTTOM_MM);
    let min_margin_pt = ((min_margin_mm * 72.0) / 25.4).round() as u32;
    decorator.set_margins(min_margin_pt);

    // Header: only show school name prominently on page 1
    let school_owned = school_name.map(|s| s.to_string());
    decorator.set_header(move |page| {
        let mut layout = elements::LinearLayout::vertical();
        if page == 1 {
            if let Some(ref s) = school_owned {
                if !s.is_empty() {
                    layout.push(
                        elements::Paragraph::new(s.clone())
                            .aligned(Alignment::Center)
                            .styled(style::Style::new().with_font_size(H2_PT).bold()),
                    );
                    layout.push(elements::Break::new(
                        (BASELINE_PT as f64 * 0.5) / BODY_PT as f64,
                    ));
                }
            }
        }
        layout.styled(style::Style::new().with_font_size((BODY_PT - 2).max(8)))
    });
    doc.set_page_decorator(decorator);

    // Styles (Style implements Copy — no need to clone)
    let h1_style = style::Style::new().with_font_size(H1_PT).bold();
    let h2_style = style::Style::new().with_font_size(H2_PT).bold();
    let h3_style = style::Style::new().with_font_size(H3_PT).bold();
    let body_style = style::Style::new().with_font_size(BODY_PT);
    let caption_style = style::Style::new().with_font_size(CAPTION_PT);

    // Start content
    doc.push(
        elements::Paragraph::new("Autorisation de sortie")
            .aligned(Alignment::Center)
            .styled(h1_style),
    );
    doc.push(elements::Break::new(
        (BASELINE_PT as f64 * 2.0) / BODY_PT as f64,
    ));

    // Build a 12-column table to place meta (grille démonstration)
    let mut table = elements::TableLayout::new(vec![1; GRID_COLS]);

    // Row: child name (first cell), rest empty
    {
        let mut row = table.row();
        let enfant = format!(
            "{} {}",
            form.enfant.nom,
            form.enfant.prenom.clone().unwrap_or_default()
        );
        row.push_element(
            elements::Paragraph::new(enfant)
                .styled(h2_style)
                .padded(Margins::trbl(0.0, 3.0, 0.0, 0.0)),
        );
        for _ in 1..GRID_COLS {
            row.push_element(elements::Paragraph::new(""));
        }
        row.push().expect("table row push");
    }

    // Second row: date / lieu positioned to the right columns
    {
        let date_str = human_date_fr(&form.date);
        let lieu = form.lieu.clone();
        let mut row = table.row();
        for col in 0..GRID_COLS {
            if col == GRID_COLS - 4 {
                row.push_element(
                    elements::Paragraph::new(format!("Date: {date_str}")).styled(body_style),
                );
            } else if col == GRID_COLS - 2 {
                row.push_element(
                    elements::Paragraph::new(format!("Lieu: {lieu}")).styled(body_style),
                );
            } else {
                row.push_element(elements::Paragraph::new(""));
            }
        }
        row.push().expect("table row push");
    }

    // push table (it will occupy full width)
    doc.push(table);
    doc.push(elements::Break::new(
        (BASELINE_PT as f64 * 1.0) / BODY_PT as f64,
    ));

    // Motif, classe, responsable
    if let Some(motif) = &form.motif {
        doc.push(elements::Paragraph::new("Motif :").styled(h3_style));
        doc.push(elements::Paragraph::new(motif.clone()).styled(body_style));
        doc.push(elements::Break::new(
            (BASELINE_PT as f64 * 0.5) / BODY_PT as f64,
        ));
    }
    if let Some(classe) = &form.classe {
        doc.push(elements::Paragraph::new(format!("Classe : {classe}")).styled(body_style));
        doc.push(elements::Break::new(
            (BASELINE_PT as f64 * 0.5) / BODY_PT as f64,
        ));
    }
    if let Some(resp) = &form.responsable {
        doc.push(
            elements::Paragraph::new(
                format!("Responsable légal : {}", resp.nom).replace("{resp}", &resp.nom),
            )
            .styled(body_style),
        );
        if let Some(tel) = &resp.telephone {
            doc.push(elements::Paragraph::new(format!("Tél : {tel}")).styled(body_style));
        }
        doc.push(elements::Break::new(
            (BASELINE_PT as f64 * 1.0) / BODY_PT as f64,
        ));
    }

    // Signature block (anchored to baseline grid)
    doc.push(
        elements::Paragraph::new("Fait à _______________________, le _______________________")
            .styled(body_style),
    );
    doc.push(elements::Break::new(
        (BASELINE_PT as f64 * 2.0) / BODY_PT as f64,
    ));
    doc.push(elements::Paragraph::new("Signature du responsable légal :").styled(body_style));
    doc.push(elements::Break::new(
        (BASELINE_PT as f64 * 2.0) / BODY_PT as f64,
    ));
    doc.push(
        elements::Paragraph::new("____________________________")
            .aligned(Alignment::Right)
            .styled(caption_style),
    );

    // Render
    doc.render_to_file(out)
        .context("échec lors du rendu PDF avec genpdf")?;
    Ok(())
}
