use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use time::Date;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Responsable {
    pub nom: String,
    pub telephone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PlageHoraire {
    pub debut: Option<String>, // HH:MM
    pub fin: Option<String>,   // HH:MM
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AutorisationForm {
    pub enfant: Enfant,
    pub date: String, // JJ/MM/AAAA
    pub lieu: String,
    pub classe: Option<String>,
    pub responsable: Option<Responsable>,
    pub plage_horaire: Option<PlageHoraire>,
    pub motif: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Enfant {
    pub nom: String,
    pub prenom: Option<String>,
}

impl AutorisationForm {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).context("read input file")?;
        if path.ends_with(".json") {
            let v = serde_json::from_str(&content).context("parse json")?;
            Ok(v)
        } else {
            // try yaml then json
            if let Ok(v) = serde_yaml::from_str(&content) {
                Ok(v)
            } else {
                let v: Self = serde_json::from_str(&content).context("parse json fallback")?;
                Ok(v)
            }
        }
    }

    pub fn from_interactive() -> Result<Self> {
        // use dialoguer to prompt
        use dialoguer::Input;
        let enfant_nom: String = Input::new()
            .with_prompt("Nom de l'enfant (required)")
            .validate_with(|input: &String| {
                if input.trim().is_empty() || input.len() > 80 {
                    Err("nom vide ou trop long (>80)")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;
        let enfant_prenom: String = Input::new()
            .with_prompt("Prénom de l'enfant (optionnel)")
            .allow_empty(true)
            .interact_text()?;
        let date: String = Input::new()
            .with_prompt("Date (JJ/MM/AAAA)")
            .validate_with(|d: &String| {
                AutorisationForm::check_date_format(d)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .interact_text()?;
        let lieu: String = Input::new()
            .with_prompt("Lieu")
            .validate_with(|s: &String| {
                if s.trim().is_empty() || s.len() > 80 {
                    Err("lieu vide ou trop long (>80)")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;
        let classe: String = Input::new()
            .with_prompt("Classe (optionnel)")
            .allow_empty(true)
            .interact_text()?;
        let responsable_nom: String = Input::new()
            .with_prompt("Nom du responsable légal (optionnel)")
            .allow_empty(true)
            .interact_text()?;
        let responsable_tel: String = Input::new()
            .with_prompt("Téléphone du responsable (optionnel)")
            .allow_empty(true)
            .interact_text()?;
        let motif: String = Input::new()
            .with_prompt("Motif (optionnel)")
            .allow_empty(true)
            .interact_text()?;
        Ok(AutorisationForm {
            enfant: Enfant {
                nom: enfant_nom,
                prenom: if enfant_prenom.is_empty() {
                    None
                } else {
                    Some(enfant_prenom)
                },
            },
            date,
            lieu,
            classe: if classe.is_empty() {
                None
            } else {
                Some(classe)
            },
            responsable: if responsable_nom.is_empty() && responsable_tel.is_empty() {
                None
            } else {
                Some(Responsable {
                    nom: responsable_nom,
                    telephone: if responsable_tel.is_empty() {
                        None
                    } else {
                        Some(responsable_tel)
                    },
                })
            },
            plage_horaire: None,
            motif: if motif.is_empty() { None } else { Some(motif) },
        })
    }

    pub fn validate(&self) -> Result<()> {
        // nom enfant present, longueur <=80
        if self.enfant.nom.trim().is_empty() || self.enfant.nom.len() > 80 {
            anyhow::bail!("Nom de l'enfant absent ou trop long");
        }
        // date valid JJ/MM/AAAA
        Self::check_date_format(&self.date).context("date invalid")?;

        // lieu
        if self.lieu.trim().is_empty() || self.lieu.len() > 80 {
            anyhow::bail!("Lieu absent ou trop long");
        }

        // responsable phone normalization/validation
        if let Some(resp) = &self.responsable {
            if let Some(tel) = &resp.telephone {
                let normalized = Self::normalize_fr_phone(tel)
                    .context("telephone format invalid (expected FR)")?;
                if normalized.len() < 8 {
                    anyhow::bail!("telephone trop court après normalisation");
                }
            }
        }
        Ok(())
    }

    fn check_date_format(s: &str) -> Result<()> {
        // parse JJ/MM/AAAA
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            anyhow::bail!("date doit être JJ/MM/AAAA");
        }
        let d = parts[0].parse::<u8>().context("jour parse failed")?;
        let m = parts[1].parse::<u8>().context("mois parse failed")?;
        let y = parts[2].parse::<i32>().context("annee parse failed")?;
        Date::from_calendar_date(y, time::Month::try_from(m).context("mois invalide")?, d)
            .context("date invalide")?;
        Ok(())
    }

    fn normalize_fr_phone(s: &str) -> Result<String> {
        // Keep only digits and plus
        let mut digits = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect::<String>();
        if digits.starts_with('0') {
            digits.remove(0);
            digits = format!("+33{digits}");
        }
        Ok(digits)
    }
}
