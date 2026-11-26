// Gestion des ressources statiques embarquées pour le Prisme d'Obsidienne (100% Rust)
// Tout le style et le runtime sont encapsulés directement dans des modules Rust natifs.

pub mod style;

use std::fs;
use std::path::Path;

pub use style::STYLE_CSS;
pub const PRISM_RUNTIME: &str = include_str!("runtime.bundle");
pub const LOGO_SVG: &str = include_str!("../../assets/logo.svg");

/// Charge le style depuis le disque s'il existe (pour le développement), sinon renvoie la constante Rust.
pub fn load_style_css<P: AsRef<Path>>(disk_path: P) -> String {
    let p = disk_path.as_ref();
    if p.exists() {
        if let Ok(content) = fs::read_to_string(p) {
            if !content.trim().is_empty() {
                return content;
            }
        }
    }
    STYLE_CSS.to_string()
}
