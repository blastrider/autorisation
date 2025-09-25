// Simple typst template — composant minimal, rempli par le binaire
#set title = "{{school_name}}"
#let header = (block(align: center) [
  heading(2, title)
])
#header

section("Autorisation de sortie")

**Enfant :** {{enfant_nom}} {{enfant_prenom}}

**Date :** {{date}}

**Lieu :** {{lieu}}

**Classe :** {{classe}}

**Responsable légal :** {{responsable_nom}}

**Téléphone :** {{responsable_tel}}

**Motif :** {{motif}}

---

Fait à ______, le ______

Signature du responsable légal : ____________________
