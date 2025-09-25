// src/lib.rs
#![forbid(unsafe_code)]

// Réexporte les modules internes pour que les tests d'intégration utilisent `autorisation::...`
pub mod domain;
pub mod infra;
pub mod render;
