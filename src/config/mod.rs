pub mod model;

use std::fs;
use std::path::{Path, PathBuf};
pub use model::Config;

pub const DEFAULT_CONFIG_FILE: &str = "config.json";

/// Résout l'emplacement optimal du fichier config.json de manière adaptative sur n'importe quel PC.
pub fn get_resolved_config_path() -> PathBuf {
    // 1. À côté du binaire exécutable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join(DEFAULT_CONFIG_FILE);
            if p.exists() {
                return p;
            }
        }
    }

    // 2. Dans ~/.gemini/config/plugins/obsidian-prism/config.json
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".gemini").join("config").join("plugins").join("obsidian-prism").join(DEFAULT_CONFIG_FILE);
        if p.exists() {
            return p;
        }
    }

    // 3. Répertoire courant
    PathBuf::from(DEFAULT_CONFIG_FILE)
}

impl Config {
    /// Charge la configuration depuis le fichier donné ou le chemin par défaut.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        if !p.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(p)
            .map_err(|e| format!("Impossible de lire le fichier de configuration '{:?}': {}", p, e))?;
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| format!("Erreur de syntaxe JSON dans '{:?}': {}", p, e))?;
        Ok(config)
    }

    /// Charge la configuration de manière adaptative.
    pub fn load() -> Self {
        Self::load_from_path(get_resolved_config_path()).unwrap_or_default()
    }

    /// Enregistre la configuration dans le fichier spécifié avec un formatage élégant.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let p = path.as_ref();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Échec de la sérialisation de la configuration: {}", e))?;
        fs::write(p, content)
            .map_err(|e| format!("Échec de l'écriture de la configuration dans '{:?}': {}", p, e))?;
        Ok(())
    }

    /// Enregistre la configuration dans le chemin résolu et synchronise la copie déployée.
    pub fn save(&self) -> Result<(), String> {
        let target = get_resolved_config_path();
        self.save_to_path(&target)?;

        // Synchronisation secondaire si déployé dans .gemini
        if let Some(home) = dirs::home_dir() {
            let gemini_cfg = home.join(".gemini").join("config").join("plugins").join("obsidian-prism").join(DEFAULT_CONFIG_FILE);
            if gemini_cfg != target && gemini_cfg.parent().map(|p| p.exists()).unwrap_or(false) {
                let _ = self.save_to_path(&gemini_cfg);
            }
        }
        Ok(())
    }

    /// Convertit la configuration en JSON compact pour injection dans le navigateur.
    pub fn to_compact_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Met à jour une valeur par sa clé textuelle.
    pub fn set_property(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "lang" | "language" => {
                self.lang = value.to_string();
                self.language = value.to_string();
            }
            "palette" => self.palette = value.to_string(),
            "lightPreset" | "light_preset" => self.light_preset = value.to_string(),
            "motionLevel" | "motion_level" => self.motion_level = value.to_string(),
            "targetFps" | "target_fps" => self.target_fps = value.to_string(),
            "telemetryModule" | "telemetry_module" => self.telemetry_module = value.to_string(),
            "effortSliderModule" | "effort_slider_module" => self.effort_slider_module = value.to_string(),
            "effortMemory" | "effort_memory" => self.effort_memory = value.to_string(),
            "periodicModelPicker" | "periodic_model_picker" => self.periodic_model_picker = value.to_string(),
            "thoughtTranslate" | "thought_translate" => self.thought_translate = value.to_string(),
            "thoughtTrigger" | "thought_trigger" => self.thought_trigger = value.to_string(),
            "mascot" => self.mascot = value.to_string(),
            "logoInk" | "logo_ink" => self.logo_ink = value.to_string(),
            "smoothCaret" | "smooth_caret" => self.smooth_caret = value == "true" || value == "1" || value == "on",
            "contextMenu" | "context_menu" => self.context_menu = value == "true" || value == "1" || value == "on",
            "codeFont" | "code_font" => self.code_font = value.to_string(),
            "ambientIntensity" | "ambient_intensity" => self.ambient_intensity = value.to_string(),
            "sidebar" => self.sidebar = value.to_string(),
            "composer" => self.composer = value.to_string(),
            "ripple" => self.ripple = value.to_string(),
            other => {
                self.extra.insert(other.to_string(), serde_json::Value::String(value.to_string()));
            }
        }
        Ok(())
    }

    /// Affiche un résumé en français dans la console.
    pub fn print_summary(&self) {
        use colored::*;
        println!("{}", "╔══════════════════════════════════════════════════════════════════════╗".cyan());
        println!("{}", "║      CONFIGURATION DU PRISME D'OBSIDIENNE (100% RUST)                ║".cyan().bold());
        println!("{}", "╠══════════════════════════════════════════════════════════════════════╣".cyan());
        println!("  {}: {}", "Langue de l'interface".bold(), self.lang.green());
        println!("  {}: {}", "Palette de couleurs".bold(), self.palette.yellow());
        println!("  {}: {}", "Thème clair alternatif".bold(), self.light_preset);
        println!("  {}: {}", "Niveau de cinétique (Ressorts)".bold(), self.motion_level);
        println!("  {}: {} FPS", "Fréquence d'affichage cible".bold(), self.target_fps);
        println!("  {}: {}", "Fiole de télémétrie 512K".bold(), self.telemetry_module);
        println!("  {}: {}", "Curseur d'effort de réflexion".bold(), self.effort_slider_module);
        println!("  {}: {}", "Mascotte Pixel".bold(), self.mascot);
        println!("  {}: {}", "Curseur fluide (Smooth Caret)".bold(), if self.smooth_caret { "Activé".green() } else { "Désactivé".red() });
        println!("  {}: {}", "Menu contextuel optique".bold(), if self.context_menu { "Activé".green() } else { "Désactivé".red() });
        println!("  {}: {}", "Module Omni intégré".bold(), self.omni_module);
        println!("{}", "╚══════════════════════════════════════════════════════════════════════╝".cyan());
    }
}
