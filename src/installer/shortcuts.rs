// Gestion et patch des raccourcis d'Antigravity pour activer le port de débogage (--remote-debugging-port)

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub const PATCHED_SHORTCUTS_FILE: &str = "patched-shortcuts.json";
pub const CDP_DEBUG_FLAG: &str = "--remote-debugging-port=9222";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchedRecord {
    pub lnk_path: String,
    pub original_arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchedStore {
    pub entries: Vec<PatchedRecord>,
}

impl PatchedStore {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(PATCHED_SHORTCUTS_FILE) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Erreur de sérialisation des raccourcis: {}", e))?;
        fs::write(PATCHED_SHORTCUTS_FILE, content)
            .map_err(|e| format!("Échec d'écriture de '{}': {}", PATCHED_SHORTCUTS_FILE, e))?;
        Ok(())
    }
}

/// Découvre les répertoires standards de raccourcis sous Windows et Linux.
pub fn get_shortcut_search_dirs() -> Vec<PathBuf> {
    let mut dirs_list = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Some(desktop) = dirs::desktop_dir() {
            dirs_list.push(desktop);
        }
        if let Ok(pub_dir) = std::env::var("PUBLIC") {
            let public_desktop = PathBuf::from(pub_dir).join("Desktop");
            if public_desktop.exists() {
                dirs_list.push(public_desktop);
            }
        } else {
            let public_desktop = PathBuf::from(r"C:\Users\Public\Desktop");
            if public_desktop.exists() {
                dirs_list.push(public_desktop);
            }
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            let start_menu = PathBuf::from(&appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs");
            if start_menu.exists() {
                dirs_list.push(start_menu);
            }

            let taskbar = PathBuf::from(&appdata)
                .join("Microsoft")
                .join("Internet Explorer")
                .join("Quick Launch")
                .join("User Pinned")
                .join("TaskBar");
            if taskbar.exists() {
                dirs_list.push(taskbar);
            }
        }

        if let Ok(progdata) = std::env::var("ProgramData") {
            let common_start = PathBuf::from(&progdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs");
            if common_start.exists() {
                dirs_list.push(common_start);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            dirs_list.push(home.join(".local").join("share").join("applications"));
        }
        dirs_list.push(PathBuf::from("/usr/share/applications"));
        dirs_list.push(PathBuf::from("/usr/local/share/applications"));
    }

    dirs_list
}

/// Résultat d'une opération de patch des raccourcis.
#[derive(Debug, Default)]
pub struct ShortcutPatchResult {
    pub scanned: usize,
    pub matched: usize,
    pub patched: Vec<String>,
    pub already_set: Vec<String>,
    pub skipped: Vec<String>,
}

/// Applique le drapeau --remote-debugging-port=9222 à tous les raccourcis Antigravity détectés.
pub fn patch_antigravity_shortcuts() -> Result<ShortcutPatchResult, String> {
    #[cfg(target_os = "windows")]
    {
        let dirs_to_search = get_shortcut_search_dirs();
        let mut store = PatchedStore::load();
        let mut result = ShortcutPatchResult::default();

        // Script PowerShell inline pour inspecter et modifier les raccourcis via WScript.Shell
        let dirs_json = serde_json::to_string(&dirs_to_search).unwrap_or_else(|_| "[]".to_string());
        let ps_script = format!(
            r#"
$dirsJson = @'
{dirs_json}
'@
$dirs = $dirsJson | ConvertFrom-Json
$ws = New-Object -ComObject WScript.Shell
$scanned = 0
$matched = 0
$patched = @()
$already = @()
$skipped = @()
$records = @()

foreach ($d in $dirs) {{
    if (-not (Test-Path -LiteralPath $d)) {{ continue }}
    $files = Get-ChildItem -LiteralPath $d -Filter "*.lnk" -Recurse -File -ErrorAction SilentlyContinue
    foreach ($f in $files) {{
        $scanned++
        try {{
            $sc = $ws.CreateShortcut($f.FullName)
            $target = [string]$sc.TargetPath
            if ($target -match '(Antigravity|Code|Cursor|VSCodium)' -and $target -match '\.exe$') {{
                $matched++
                $args = [string]$sc.Arguments
                if ($args -notmatch '--remote-debugging-port') {{
                    $newArgs = if ($args) {{ "$args --remote-debugging-port=9222" }} else {{ "--remote-debugging-port=9222" }}
                    $records += @{{ lnkPath = $f.FullName; originalArgs = $args }}
                    $sc.Arguments = $newArgs
                    $sc.Save()
                    $patched += $f.FullName
                }} else {{
                    $already += $f.FullName
                }}
            }}
        }} catch {{
            $skipped += $f.FullName
        }}
    }}
}}

[PSCustomObject]@{{
    scanned = $scanned
    matched = $matched
    patched = $patched
    already = $already
    skipped = $skipped
    records = $records
}} | ConvertTo-Json -Compress
"#,
            dirs_json = dirs_json
        );

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-NonInteractive", "-Command", &ps_script])
            .output()
            .map_err(|e| format!("Échec de l'exécution de PowerShell: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Erreur PowerShell lors du patch des raccourcis: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
            .map_err(|e| format!("Réponse JSON invalide de PowerShell: {}. Sortie: {}", e, stdout))?;

        result.scanned = parsed["scanned"].as_u64().unwrap_or(0) as usize;
        result.matched = parsed["matched"].as_u64().unwrap_or(0) as usize;

        if let Some(arr) = parsed["patched"].as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    result.patched.push(s.to_string());
                }
            }
        }
        if let Some(arr) = parsed["already"].as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    result.already_set.push(s.to_string());
                }
            }
        }
        if let Some(arr) = parsed["skipped"].as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    result.skipped.push(s.to_string());
                }
            }
        }

        if let Some(recs) = parsed["records"].as_array() {
            for r in recs {
                let lnk = r["lnkPath"].as_str().unwrap_or("").to_string();
                let orig = r["originalArgs"].as_str().unwrap_or("").to_string();
                if !lnk.is_empty() {
                    store.entries.push(PatchedRecord {
                        lnk_path: lnk,
                        original_arguments: orig,
                    });
                }
            }
            let _ = store.save();
        }

        Ok(result)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(ShortcutPatchResult::default())
    }
}

/// Restaure les raccourcis dans leur état d'origine avant le patch.
pub fn restore_antigravity_shortcuts() -> Result<usize, String> {
    #[cfg(target_os = "windows")]
    {
        let store = PatchedStore::load();
        if store.entries.is_empty() {
            return Ok(0);
        }

        let store_json = serde_json::to_string(&store.entries).unwrap_or_else(|_| "[]".to_string());
        let ps_script = format!(
            r#"
$entriesJson = @'
{store_json}
'@
$entries = $entriesJson | ConvertFrom-Json
$ws = New-Object -ComObject WScript.Shell
$restored = 0

foreach ($e in $entries) {{
    $p = [string]$e.lnk_path
    $orig = [string]$e.original_arguments
    if (Test-Path -LiteralPath $p) {{
        try {{
            $sc = $ws.CreateShortcut($p)
            $sc.Arguments = $orig
            $sc.Save()
            $restored++
        }} catch {{}}
    }}
}}
Write-Output $restored
"#,
            store_json = store_json
        );

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-NonInteractive", "-Command", &ps_script])
            .output()
            .map_err(|e| format!("Échec d'exécution de PowerShell: {}", e))?;

        let count: usize = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);

        let _ = fs::remove_file(PATCHED_SHORTCUTS_FILE);
        Ok(count)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(0)
    }
}
