// Manipulation native et 100% Rust des archives Electron ASAR
// Permet de lire, extraire et réemballer un fichier app.asar sans dépendre de Node.js ni d'outils externes.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const PATCH_START_MARKER: &str = "/* __AG_PRISM_START__ */";
pub const PATCH_END_MARKER: &str = "/* __AG_PRISM_END__ */";

pub struct AsarArchive {
    pub header: Value,
    pub payload: Vec<u8>,
}

impl AsarArchive {
    /// Lit et analyse une archive ASAR depuis un buffer binaire.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("Fichier ASAR trop court ou corrompu.".to_string());
        }

        // Lecture des entêtes Pickle (Little Endian)
        let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        if magic != 4 {
            return Err(format!("Signature ASAR invalide (reçu {}, attendu 4)", magic));
        }

        let header_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let json_len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;

        if bytes.len() < 16 + json_len {
            return Err("Entête ASAR tronqué.".to_string());
        }

        let header_json_str = std::str::from_utf8(&bytes[16..16 + json_len])
            .map_err(|e| format!("Entête JSON ASAR non UTF-8: {}", e))?;

        let header: Value = serde_json::from_str(header_json_str)
            .map_err(|e| format!("JSON invalide dans l'entête ASAR: {}", e))?;

        let data_start = 8 + header_size;
        if bytes.len() < data_start {
            return Err("Données du payload ASAR hors limites.".to_string());
        }

        let payload = bytes[data_start..].to_vec();

        Ok(Self { header, payload })
    }

    /// Extrait le contenu d'un fichier stocké dans l'ASAR (ex: "dist/preload.js").
    pub fn read_file(&self, path_in_asar: &str) -> Result<Vec<u8>, String> {
        let parts: Vec<&str> = path_in_asar.split('/').collect();
        let mut cur = &self.header;

        for part in parts {
            cur = &cur["files"][part];
            if cur.is_null() {
                return Err(format!("Chemin '{}' introuvable dans l'ASAR.", path_in_asar));
            }
        }

        if cur["unpacked"].as_bool() == Some(true) {
            return Err(format!("Le fichier '{}' est déballé (unpacked).", path_in_asar));
        }

        let size = cur["size"]
            .as_u64()
            .ok_or_else(|| format!("Taille manquante pour '{}'", path_in_asar))? as usize;

        let offset_str = cur["offset"]
            .as_str()
            .ok_or_else(|| format!("Décalage manquant pour '{}'", path_in_asar))?;

        let offset: usize = offset_str
            .parse()
            .map_err(|e| format!("Décalage invalide '{}': {}", offset_str, e))?;

        if offset + size > self.payload.len() {
            return Err(format!("Plage de données hors limites pour '{}'", path_in_asar));
        }

        Ok(self.payload[offset..offset + size].to_vec())
    }

    /// Réemballe l'ASAR avec des remplacements de fichiers spécifiques (ex: ["dist/preload.js" -> new_content]).
    pub fn repack(&self, replacements: &HashMap<String, Vec<u8>>) -> Result<Vec<u8>, String> {
        let mut new_header = self.header.clone();
        let mut new_payload = Vec::new();

        // Réécriture récursive de l'arborescence des fichiers
        repack_node(&mut new_header, &self.payload, "", replacements, &mut new_payload)?;

        // Construction de l'entête binaire ASAR
        let header_json = serde_json::to_string(&new_header)
            .map_err(|e| format!("Erreur de sérialisation JSON: {}", e))?;
        let json_bytes = header_json.as_bytes();
        let json_len = json_bytes.len();
        let aligned_json_len = (json_len + 3) / 4 * 4;
        let header_size = aligned_json_len + 8;

        let mut out = Vec::with_capacity(16 + aligned_json_len + new_payload.len());
        out.extend_from_slice(&(4u32).to_le_bytes());
        out.extend_from_slice(&(header_size as u32).to_le_bytes());
        out.extend_from_slice(&((header_size - 4) as u32).to_le_bytes());
        out.extend_from_slice(&(json_len as u32).to_le_bytes());
        out.extend_from_slice(json_bytes);

        // Remplissage avec des octets nuls pour aligner sur 4 octets
        for _ in 0..(aligned_json_len - json_len) {
            out.push(0);
        }

        // Concaténation du payload
        out.extend_from_slice(&new_payload);

        Ok(out)
    }
}

fn repack_node(
    node: &mut Value,
    orig_payload: &[u8],
    prefix: &str,
    replacements: &HashMap<String, Vec<u8>>,
    out_payload: &mut Vec<u8>,
) -> Result<(), String> {
    if let Some(files_map) = node.get_mut("files").and_then(|f| f.as_object_mut()) {
        for (name, child) in files_map.iter_mut() {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if child.get("files").is_some() {
                repack_node(child, orig_payload, &path, replacements, out_payload)?;
            } else if child.get("unpacked").and_then(|u| u.as_bool()) == Some(true) {
                // Fichier externe déballé, conserver les métadonnées
            } else if let (Some(size), Some(offset_str)) = (child.get("size").and_then(|s| s.as_u64()), child.get("offset").and_then(|o| o.as_str())) {
                let current_offset = out_payload.len();
                let file_data = if let Some(replacement) = replacements.get(&path) {
                    replacement.clone()
                } else {
                    let old_off: usize = offset_str.parse().unwrap_or(0);
                    let old_sz = size as usize;
                    if old_off + old_sz <= orig_payload.len() {
                        orig_payload[old_off..old_off + old_sz].to_vec()
                    } else {
                        Vec::new()
                    }
                };

                child["size"] = json!(file_data.len());
                child["offset"] = json!(current_offset.to_string());
                out_payload.extend_from_slice(&file_data);
            }
        }
    }
    Ok(())
}

/// Supprime les blocs délimités par les marqueurs de patch.
pub fn strip_marker_block(source: &str, start_marker: &str, end_marker: &str) -> String {
    let mut out = source.to_string();
    while let Some(start_idx) = out.find(start_marker) {
        if let Some(end_rel_idx) = out[start_idx..].find(end_marker) {
            let end_idx = start_idx + end_rel_idx + end_marker.len();
            out.replace_range(start_idx..end_idx, "");
        } else {
            out.truncate(start_idx);
            break;
        }
    }
    out
}

/// Génère le snippet d'injection pour le preload d'Antigravity.
pub fn generate_patched_preload(original_preload: &str, inject_code: &str) -> String {
    let clean_base = strip_marker_block(original_preload, PATCH_START_MARKER, PATCH_END_MARKER);
    let clean_trimmed = clean_base.trim_end();
    let code_literal = serde_json::to_string(inject_code).unwrap_or_else(|_| "\"\"".to_string());

    format!(
        "{}\n\n{}\ntry {{\n  const __agPrismCode = {};\n  const __agPrismInjectMainWorld = () => {{\n    try {{\n      const wf =\n        (typeof electron_1 !== 'undefined' && electron_1 && electron_1.webFrame) ||\n        (typeof require === 'function' ? require('electron')?.webFrame : null);\n      if (wf && typeof wf.executeJavaScript === 'function') {{\n        void wf.executeJavaScript(__agPrismCode);\n      }}\n    }} catch (err) {{\n      console.error('[Prisme d\\'Obsidienne] webFrame executeJavaScript error:', err);\n    }}\n  }};\n  if (typeof window !== 'undefined' && typeof document !== 'undefined') {{\n    if (document.readyState === 'loading') {{\n      window.addEventListener('DOMContentLoaded', __agPrismInjectMainWorld, {{ once: true }});\n    }} else {{\n      __agPrismInjectMainWorld();\n    }}\n  }}\n}} catch (err) {{\n  console.error('[Prisme d\\'Obsidienne] Erreur bootstrap preload:', err);\n}}\n{}\n",
        clean_trimmed,
        PATCH_START_MARKER,
        code_literal,
        PATCH_END_MARKER
    )
}

/// Applique le patch persistant dans app.asar en 100% Rust.
pub fn patch_asar_file<P: AsRef<Path>>(asar_path: P, inject_code: &str) -> Result<(), String> {
    let p = asar_path.as_ref();
    if !p.exists() {
        return Err(format!("Fichier introuvable: {:?}", p));
    }

    let bytes = fs::read(p).map_err(|e| format!("Échec de lecture de '{:?}': {}", p, e))?;
    let asar = AsarArchive::parse(&bytes)?;

    // Sauvegarde de sécurité
    let backup_path = p.with_extension("asar.prism.bak");
    if !backup_path.exists() {
        let _ = fs::write(&backup_path, &bytes);
    }

    let preload_bytes = asar.read_file("dist/preload.js")?;
    let original_preload = String::from_utf8(preload_bytes)
        .map_err(|e| format!("dist/preload.js n'est pas un texte UTF-8 valide: {}", e))?;

    let patched_preload = generate_patched_preload(&original_preload, inject_code);
    let mut replacements = HashMap::new();
    replacements.insert("dist/preload.js".to_string(), patched_preload.into_bytes());

    let repacked_bytes = asar.repack(&replacements)?;
    fs::write(p, repacked_bytes)
        .map_err(|e| format!("Échec d'écriture dans '{:?}': {}", p, e))?;

    Ok(())
}

/// Restaure le fichier app.asar dans son état d'origine en 100% Rust.
pub fn restore_asar_file<P: AsRef<Path>>(asar_path: P) -> Result<bool, String> {
    let p = asar_path.as_ref();
    let backup_path = p.with_extension("asar.prism.bak");

    if backup_path.exists() {
        fs::copy(&backup_path, p)
            .map_err(|e| format!("Échec de restauration depuis '{:?}': {}", backup_path, e))?;
        let _ = fs::remove_file(backup_path);
        return Ok(true);
    }

    // Si pas de fichier .bak, on déplie et retire le marqueur
    if p.exists() {
        let bytes = fs::read(p).map_err(|e| format!("Échec de lecture: {}", e))?;
        if let Ok(asar) = AsarArchive::parse(&bytes) {
            if let Ok(preload_bytes) = asar.read_file("dist/preload.js") {
                if let Ok(text) = String::from_utf8(preload_bytes) {
                    if text.contains(PATCH_START_MARKER) {
                        let cleaned = strip_marker_block(&text, PATCH_START_MARKER, PATCH_END_MARKER);
                        let mut replacements = HashMap::new();
                        replacements.insert("dist/preload.js".to_string(), cleaned.into_bytes());
                        if let Ok(repacked) = asar.repack(&replacements) {
                            let _ = fs::write(p, repacked);
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}
