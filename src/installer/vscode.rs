// Détection et intégration pour l'extension Antigravity dans Visual Studio Code, Antigravity IDE, Cursor et VSCodium
// Permet à Obsidian_Rs de fonctionner aussi bien dans l'extension VS Code que dans Antigravity IDE et l'IDE autonome.

use std::fs;
use std::path::PathBuf;

pub const VSCODE_EXTENSION_PATCH_MARKER: &str = "/* __AG_PRISM_VSCODE_START__ */";
pub const VSCODE_EXTENSION_PATCH_END: &str = "/* __AG_PRISM_VSCODE_END__ */";

#[derive(Debug, Clone)]
pub struct VsCodeExtensionInfo {
    pub name: String,
    pub path: PathBuf,
    pub bridge_js: PathBuf,
    pub is_patched: bool,
}

/// Recherche tous les répertoires d'extensions Antigravity (VS Code, Antigravity IDE, Cursor, VSCodium).
pub fn find_antigravity_vscode_extensions() -> Vec<VsCodeExtensionInfo> {
    let mut found = Vec::new();

    let mut ext_roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        ext_roots.push(home.join(".vscode").join("extensions"));
        ext_roots.push(home.join(".cursor").join("extensions"));
        ext_roots.push(home.join(".vscode-insiders").join("extensions"));
        ext_roots.push(home.join(".vscodium").join("extensions"));
        ext_roots.push(home.join(".antigravity-ide").join("extensions"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app) = dirs::data_local_dir() {
            ext_roots.push(local_app.join("Programs").join("Antigravity IDE").join("resources").join("app").join("extensions"));
            ext_roots.push(local_app.join("Programs").join("antigravity-ide").join("resources").join("app").join("extensions"));
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            ext_roots.push(PathBuf::from(prog_files).join("Antigravity IDE").join("resources").join("app").join("extensions"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        ext_roots.push(PathBuf::from("/Applications/Antigravity IDE.app/Contents/Resources/app/extensions"));
    }

    #[cfg(target_os = "linux")]
    {
        ext_roots.push(PathBuf::from("/opt/antigravity-ide/resources/app/extensions"));
        ext_roots.push(PathBuf::from("/usr/lib/antigravity-ide/resources/app/extensions"));
    }

    for root in ext_roots {
        if !root.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let dir_lower = dir_name.to_lowercase();
                    if dir_lower.contains("antigravity") || dir_lower.contains("gemini") || dir_lower == "agy" {
                        let bridge = path.join("bridge.js");
                        if bridge.exists() {
                            let is_patched = fs::read_to_string(&bridge)
                                .map(|s| s.contains(VSCODE_EXTENSION_PATCH_MARKER))
                                .unwrap_or(false);

                            found.push(VsCodeExtensionInfo {
                                name: dir_name,
                                path,
                                bridge_js: bridge,
                                is_patched,
                            });
                        }
                    }
                }
            }
        }
    }

    found
}

/// Applique le patch dans le script bridge.js de l'extension VS Code pour injecter les styles et le runtime.
pub fn patch_vscode_extension(ext: &VsCodeExtensionInfo, inject_code: &str) -> Result<bool, String> {
    if !ext.bridge_js.exists() {
        return Err(format!("Fichier bridge.js introuvable dans '{:?}'", ext.bridge_js));
    }

    let original = fs::read_to_string(&ext.bridge_js)
        .map_err(|e| format!("Impossible de lire '{:?}': {}", ext.bridge_js, e))?;

    // Création d'une sauvegarde
    let backup_path = ext.bridge_js.with_extension("js.prism.bak");
    if !backup_path.exists() {
        let _ = fs::write(&backup_path, &original);
    }

    // Nettoyage de l'ancien patch s'il existe
    let mut cleaned = original;
    while let Some(start) = cleaned.find(VSCODE_EXTENSION_PATCH_MARKER) {
        if let Some(end_rel) = cleaned[start..].find(VSCODE_EXTENSION_PATCH_END) {
            let end = start + end_rel + VSCODE_EXTENSION_PATCH_END.len();
            cleaned.replace_range(start..end, "");
        } else {
            cleaned.truncate(start);
            break;
        }
    }

    let code_literal = serde_json::to_string(inject_code).unwrap_or_else(|_| "\"\"".to_string());
    let patch_snippet = format!(
        r#"
{}
try {{
  const __agPrismCode = {};
  const __injectIntoIframe = () => {{
    try {{
      const frame = document.getElementById("jetski-frame");
      if (frame && frame.contentWindow) {{
        const fDoc = frame.contentDocument || frame.contentWindow.document;
        if (fDoc && (fDoc.readyState === "complete" || fDoc.readyState === "interactive")) {{
          if (!fDoc.getElementById("ag-prism-css")) {{
            const s = fDoc.createElement("script");
            s.id = "ag-prism-runtime-loader";
            s.textContent = __agPrismCode;
            (fDoc.head || fDoc.documentElement).appendChild(s);
          }}
        }}
      }}
    }} catch (e) {{}}
  }};
  if (typeof window !== "undefined") {{
    window.addEventListener("message", __injectIntoIframe);
    setInterval(__injectIntoIframe, 1200);
  }}
}} catch (err) {{}}
{}
"#,
        VSCODE_EXTENSION_PATCH_MARKER,
        code_literal,
        VSCODE_EXTENSION_PATCH_END
    );

    let final_content = format!("{}\n{}", cleaned.trim_end(), patch_snippet);
    fs::write(&ext.bridge_js, final_content)
        .map_err(|e| format!("Échec d'écriture dans '{:?}': {}", ext.bridge_js, e))?;

    Ok(true)
}

/// Restaure le fichier bridge.js de l'extension VS Code dans son état d'origine.
pub fn restore_vscode_extension(ext: &VsCodeExtensionInfo) -> Result<bool, String> {
    let backup_path = ext.bridge_js.with_extension("js.prism.bak");
    if backup_path.exists() {
        fs::copy(&backup_path, &ext.bridge_js)
            .map_err(|e| format!("Échec de restauration depuis '{:?}': {}", backup_path, e))?;
        let _ = fs::remove_file(backup_path);
        return Ok(true);
    }

    if ext.bridge_js.exists() {
        if let Ok(mut text) = fs::read_to_string(&ext.bridge_js) {
            if text.contains(VSCODE_EXTENSION_PATCH_MARKER) {
                while let Some(start) = text.find(VSCODE_EXTENSION_PATCH_MARKER) {
                    if let Some(end_rel) = text[start..].find(VSCODE_EXTENSION_PATCH_END) {
                        let end = start + end_rel + VSCODE_EXTENSION_PATCH_END.len();
                        text.replace_range(start..end, "");
                    } else {
                        text.truncate(start);
                        break;
                    }
                }
                let _ = fs::write(&ext.bridge_js, text);
                return Ok(true);
            }
        }
    }

    Ok(false)
}
