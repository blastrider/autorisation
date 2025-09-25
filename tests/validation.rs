use autorisation::domain::form::AutorisationForm;

#[test]
fn valid_form_parses_and_validates() {
    let f = AutorisationForm {
        enfant: autorisation::domain::form::Enfant {
            nom: "Dupont".into(),
            prenom: Some("Jean".into()),
        },
        date: "25/09/2025".into(),
        lieu: "Rennes".into(),
        classe: Some("CM1".into()),
        responsable: None,
        plage_horaire: None,
        motif: None,
    };
    assert!(f.validate().is_ok());
}

#[test]
fn invalid_date_fails() {
    let f = AutorisationForm {
        enfant: autorisation::domain::form::Enfant {
            nom: "X".into(),
            prenom: None,
        },
        date: "31/02/2025".into(),
        lieu: "X".into(),
        classe: None,
        responsable: None,
        plage_horaire: None,
        motif: None,
    };
    assert!(f.validate().is_err());
}
