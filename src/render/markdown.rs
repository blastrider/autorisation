use crate::domain::form::AutorisationForm;
use crate::domain::format::human_date_fr;
use anyhow::Result;

pub fn render_markdown(form: &AutorisationForm, school_name: Option<&str>) -> Result<String> {
    let mut s = String::new();
    if let Some(school) = school_name {
        s.push_str(&format!("# {school}\n\n"));
    }
    s.push_str("## Autorisation de sortie\n\n");
    s.push_str(&format!(
        "**Enfant :** {} {}\n\n",
        form.enfant.nom,
        form.enfant.prenom.clone().unwrap_or_default()
    ));
    s.push_str(&format!("**Date :** {}\n\n", human_date_fr(&form.date)));
    s.push_str(&format!("**Lieu :** {}\n\n", form.lieu));
    if let Some(classe) = &form.classe {
        s.push_str(&format!("**Classe :** {classe}\n\n"));
    }
    if let Some(resp) = &form.responsable {
        s.push_str(&format!("**Responsable légal :** {}\n\n", resp.nom));
        if let Some(t) = &resp.telephone {
            s.push_str(&format!("**Tél :** {t}\n\n"));
        }
    }
    if let Some(m) = &form.motif {
        s.push_str(&format!("**Motif :** {m}\n\n"));
    }
    s.push_str(
        "\n\nFait à ____, le ____\n\n\nSignature du responsable légal : ___________________\n",
    );
    Ok(s)
}
