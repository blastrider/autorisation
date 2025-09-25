# Autorisation — CLI (offline)

But : générer une **autorisation de sortie** A4 (PDF) propre à signer, à partir d'un YAML/JSON ou via prompts interactifs.

## Installation
- Rust toolchain (stable). Recommandé : `rustup update stable` (Rust 1.90+ recommandé). :contentReference[oaicite:5]{index=5}
- `typst` (optionnel, pour rendu typographique premium). Si absent, un fallback PDF minimal est produit.
- Build : `cargo build --release`

## Usage
- `cargo run -- --input examples/autorisation.yml --out autorisation_sortie.pdf`
- `cargo run -- --interactive --out autorisation_interactive.pdf`
- `cargo run -- --input examples/autorisation.yml --md autorisation.md`

## Remarques importantes
- YAML supporte `serde_yaml` mais ce crate a été marqué comme **deprecated** par son auteur ; JSON est recommandé pour long terme. Le binaire accepte YAML pour compatibilité. :contentReference[oaicite:6]{index=6}
- Rendu PDF « premium » : si `typst` est installé, le binaire génère un `.typ` et appelle `typst compile`; sinon fallback minimal garanti. Pour mise en page riche, installez `typst` (voir https://typst.org).
- Logs : `RUST_LOG=info cargo run ...` active logs (tracing + EnvFilter).
- License : MIT/Apache-2.0.

## Exemples copiables
- `autorisation --input examples/autorisation.yml --out autorisation_sortie.pdf`
