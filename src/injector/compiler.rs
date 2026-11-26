// Compilateur du payload d'injection pour Antigravity (100% Rust)
// Assemble les modules et la feuille de style CSS dans une expression IIFE autonome et sécurisée.

use crate::assets::*;
use crate::config::Config;
use sha2::{Digest, Sha256};

pub const STYLE_ID: &str = "ag-prism-css";
pub const SETTING_BINDING: &str = "__agPrismSetting";

/// Calcule l'empreinte SHA-256 hexadécimale d'une chaîne UTF-8.
pub fn compute_sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Génère l'expression JavaScript complète à injecter dans les fenêtres d'Antigravity.
pub fn build_inject_expression(config: &Config, custom_css: Option<&str>) -> String {
    let css = custom_css.unwrap_or(STYLE_CSS);
    let css_json = serde_json::to_string(css).unwrap_or_else(|_| "\"\"".to_string());
    let config_json = config.to_compact_json();
    let css_hash = compute_sha256(css);

    format!(
        r#"(function() {{
    const W = window, D = document, GT = (typeof globalThis !== 'undefined' ? globalThis : window);
    const SID = "{style_id}";
    const CSS_HASH = "{css_hash}";
    const CONFIG = {config_json};

    // 1. Application ou mise à jour de la feuille de style CSS du Prisme
    try {{
        let st = D.getElementById(SID);
        if (!st) {{
            st = D.createElement('style');
            st.id = SID;
            (D.head || D.documentElement).appendChild(st);
        }}
        if (st.getAttribute('data-ag-cs-css-hash') !== CSS_HASH || st.textContent.length === 0) {{
            st.textContent = {css_json};
            st.setAttribute('data-ag-cs-css-hash', CSS_HASH);
        }}
    }} catch (e) {{
        console.error('[Prisme d\'Obsidienne] Échec de l\'application CSS:', e);
    }}

    // 2. Définition et exécution du runtime du Prisme d'Obsidienne (100% Rust)
    try {{
        {runtime_bundle}

        // 3. Initialisation de l'interface
        if (typeof GT.__agCsUiMain === 'function') {{
            GT.__agCsUiMain(CONFIG);
        }} else if (typeof W.__agCsUiMain === 'function') {{
            W.__agCsUiMain(CONFIG);
        }}

        // 4. Pont de rechargement à chaud (Hot-Reload)
        W.__agPrismHotReload = {{
            updateCss: function(newCss) {{
                const el = D.getElementById(SID);
                if (el) el.textContent = newCss;
            }},
            updateConfig: function(newCfg) {{
                if (W.__agPrism && typeof W.__agPrism.refreshConfig === 'function') {{
                    W.__agPrism.refreshConfig(newCfg);
                }}
            }}
        }};

        console.log('[Obsidian_Rs] v1.0.8 (100% Rust) :: Dev: Edmond NOUMEGNI << Hacker Éthique >> :: https://portfolio-nine-jade-22.vercel.app/');
        return {{ ok: true, version: "1.0.8", statut: "injecte", langue: CONFIG.lang || "fr" }};
    }} catch (err) {{
        console.error('[Prisme d\'Obsidienne] Erreur lors de l\'initialisation du runtime:', err);
        return {{ ok: false, erreur: String(err && err.message ? err.message : err) }};
    }}
}})();"#,
        style_id = STYLE_ID,
        css_hash = css_hash,
        config_json = config_json,
        css_json = css_json,
        runtime_bundle = PRISM_RUNTIME,
    )
}

/// Génère l'expression JavaScript de suppression / désinstallation pour restaurer la fenêtre dans son état natif.
pub fn build_remove_expression() -> &'static str {
    r#"(() => {
    try {
        // Suppression du style injecté
        document.getElementById('ag-prism-css')?.remove();
        document.getElementById('ag-claude-style-css')?.remove();

        // Nettoyage de l'instance runtime
        if (window.__agPrism && typeof window.__agPrism.destroy === 'function') {
            window.__agPrism.destroy();
        }
        if (window.__agClaudeStyle && typeof window.__agClaudeStyle.destroy === 'function') {
            window.__agClaudeStyle.destroy();
        }

        delete window.__agPrism;
        delete globalThis.__agPrism;
        delete window.__agClaudeStyle;
        delete globalThis.__agClaudeStyle;
        delete window.__agPrismHotReload;

        // Suppression des éléments DOM résiduels (mascottes, fioles, panneaux)
        document.querySelectorAll('.ag-cs-mascot-rig, .ag-prism-vial-dock, .ag-prism-settings-modal, .ag-cs-context-menu').forEach(el => el.remove());

        console.log('[Prisme d\'Obsidienne] Plugin désactivé et interface nettoyée.');
        return { ok: true, statut: "desactive" };
    } catch (err) {
        return { ok: false, erreur: String(err) };
    }
})()"#
}

/// Génère l'expression JavaScript de mise à jour à chaud de la configuration.
pub fn build_config_refresh_expression(config: &Config) -> String {
    let json = config.to_compact_json();
    format!(
        r#"(() => {{
    try {{
        const cfg = {};
        if (window.__agPrism && typeof window.__agPrism.refreshConfig === 'function') {{
            window.__agPrism.refreshConfig(cfg);
        }}
        if (window.__agPrismHotReload && typeof window.__agPrismHotReload.updateConfig === 'function') {{
            window.__agPrismHotReload.updateConfig(cfg);
        }}
        return {{ ok: true, statut: "recharge" }};
    }} catch (e) {{
        return {{ ok: false, erreur: String(e) }};
    }}
}})()"#,
        json
    )
}

/// Génère l'expression JavaScript de mise à jour à chaud de la feuille de style CSS.
pub fn build_css_refresh_expression(new_css: &str) -> String {
    let css_json = serde_json::to_string(new_css).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"(() => {{
    try {{
        const el = document.getElementById('{style_id}');
        if (el) {{
            el.textContent = {css_json};
            return {{ ok: true, statut: "css_mis_a_jour" }};
        }}
        return {{ ok: false, raison: "balise_style_introuvable" }};
    }} catch (e) {{
        return {{ ok: false, erreur: String(e) }};
    }}
}})()"#,
        style_id = STYLE_ID,
        css_json = css_json
    )
}
