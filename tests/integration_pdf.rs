use autorisation::domain::form::AutorisationForm;
use autorisation::render::pdf;
use std::path::PathBuf;

#[test]
fn generate_pdf_integration() {
    let f = AutorisationForm {
        enfant: autorisation::domain::form::Enfant {
            nom: "Izi".into(),
            prenom: Some("Test".into()),
        },
        date: "25/09/2025".into(),
        lieu: "Chez moi".into(),
        classe: None,
        responsable: None,
        plage_horaire: None,
        motif: None,
    };
    let mut out = PathBuf::from(std::env::temp_dir());
    out.push("autorisation_ci_test.pdf");
    let _ = std::fs::remove_file(&out);
    let r = pdf::render_pdf(&f, None, &out);
    assert!(r.is_ok());
    let md = std::fs::metadata(&out).expect("pdf produced");
    assert!(md.len() > 0);
}
