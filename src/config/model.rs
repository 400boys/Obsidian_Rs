// Modèle de configuration du Prisme d'Obsidienne (100% Rust)
// Gère la persistance et la validation des options avec le français comme langue par défaut.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_fr")]
    pub language: String,

    #[serde(default = "default_fr")]
    pub lang: String,

    #[serde(default = "default_palette")]
    pub palette: String,

    #[serde(default = "default_light_preset")]
    pub light_preset: String,

    #[serde(default = "default_motion_level")]
    pub motion_level: String,

    #[serde(default = "default_target_fps")]
    pub target_fps: String,

    #[serde(default = "default_auto")]
    pub telemetry_module: String,

    #[serde(default = "default_on")]
    pub effort_slider_module: String,

    #[serde(default = "default_on")]
    pub effort_memory: String,

    #[serde(default = "default_on")]
    pub periodic_model_picker: String,

    #[serde(default = "default_off")]
    pub thought_translate: String,

    #[serde(default = "default_preload")]
    pub thought_trigger: String,

    #[serde(default = "default_studio")]
    pub home_layout: String,

    #[serde(default = "default_studio")]
    pub home_mode: String,

    #[serde(default = "default_true")]
    pub home_ripple: bool,

    #[serde(default = "default_off")]
    pub conv_ripple: String,

    #[serde(default = "default_on")]
    pub ripple: String,

    #[serde(default = "default_true")]
    pub composer_restyle: bool,

    #[serde(default = "default_on")]
    pub composer: String,

    #[serde(default = "default_studio")]
    pub placeholder_style: String,

    #[serde(default = "default_studio")]
    pub prompt_style: String,

    #[serde(default = "default_mascot")]
    pub mascot: String,

    #[serde(default = "default_logo_ink")]
    pub logo_ink: String,

    #[serde(default = "default_true")]
    pub sidebar_restyle: bool,

    #[serde(default = "default_on")]
    pub sidebar: String,

    #[serde(default)]
    pub sidebar_compact: bool,

    #[serde(default = "default_off")]
    pub sidebar_nav_fold: String,

    #[serde(default = "default_true")]
    pub smooth_caret: bool,

    #[serde(default = "default_smooth")]
    pub scroll_bottom_motion: String,

    #[serde(default = "default_true")]
    pub context_menu: bool,

    #[serde(default = "default_accent_tone")]
    pub accent_tone: String,

    #[serde(default = "default_corner_style")]
    pub corner_style: String,

    #[serde(default = "default_code_font")]
    pub code_font: String,

    #[serde(default)]
    pub code_ligatures: bool,

    #[serde(default = "default_ambient_intensity")]
    pub ambient_intensity: String,

    #[serde(default = "default_thought_view")]
    pub thought_view: String,

    #[serde(default = "default_on")]
    pub omni_module: String,

    #[serde(default = "default_link_target")]
    pub link_target: String,

    #[serde(default = "default_off")]
    pub sidebar_studio: String,

    #[serde(default = "default_auto")]
    pub host_localization: String,

    #[serde(default = "default_off")]
    pub auto_update: String,

    #[serde(default = "default_update_channel")]
    pub update_channel: String,

    #[serde(default)]
    pub auto_archive_on_exit: bool,

    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_fr() -> String { "fr".to_string() }
fn default_palette() -> String { "obsidian".to_string() }
fn default_light_preset() -> String { "crystal".to_string() }
fn default_motion_level() -> String { "ultra".to_string() }
fn default_target_fps() -> String { "180".to_string() }
fn default_auto() -> String { "auto".to_string() }
fn default_on() -> String { "on".to_string() }
fn default_off() -> String { "off".to_string() }
fn default_preload() -> String { "preload".to_string() }
fn default_studio() -> String { "studio".to_string() }
fn default_true() -> bool { true }
fn default_mascot() -> String { "crab".to_string() }
fn default_logo_ink() -> String { "accent".to_string() }
fn default_smooth() -> String { "smooth".to_string() }
fn default_accent_tone() -> String { "follow".to_string() }
fn default_corner_style() -> String { "balanced".to_string() }
fn default_code_font() -> String { "default".to_string() }
fn default_ambient_intensity() -> String { "100".to_string() }
fn default_thought_view() -> String { "immersive".to_string() }
fn default_link_target() -> String { "builtin".to_string() }
fn default_update_channel() -> String { "stable".to_string() }
fn default_poll_interval() -> u64 { 2500 }

impl Default for Config {
    fn default() -> Self {
        Self {
            language: default_fr(),
            lang: default_fr(),
            palette: default_palette(),
            light_preset: default_light_preset(),
            motion_level: default_motion_level(),
            target_fps: default_target_fps(),
            telemetry_module: default_auto(),
            effort_slider_module: default_auto(),
            effort_memory: default_on(),
            periodic_model_picker: default_on(),
            thought_translate: default_off(),
            thought_trigger: default_preload(),
            home_layout: default_studio(),
            home_mode: default_studio(),
            home_ripple: default_true(),
            conv_ripple: default_off(),
            ripple: default_on(),
            composer_restyle: default_true(),
            composer: default_on(),
            placeholder_style: default_studio(),
            prompt_style: default_studio(),
            mascot: default_mascot(),
            logo_ink: default_logo_ink(),
            sidebar_restyle: default_true(),
            sidebar: default_on(),
            sidebar_compact: false,
            sidebar_nav_fold: default_off(),
            smooth_caret: default_true(),
            scroll_bottom_motion: default_smooth(),
            context_menu: default_true(),
            accent_tone: default_accent_tone(),
            corner_style: default_corner_style(),
            code_font: default_code_font(),
            code_ligatures: false,
            ambient_intensity: default_ambient_intensity(),
            thought_view: default_thought_view(),
            omni_module: default_on(),
            link_target: default_link_target(),
            sidebar_studio: default_off(),
            host_localization: default_auto(),
            auto_update: default_off(),
            update_channel: default_update_channel(),
            auto_archive_on_exit: false,
            poll_interval_ms: default_poll_interval(),
            extra: HashMap::new(),
        }
    }
}
