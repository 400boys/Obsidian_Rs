// Tests d'intégration et unitaires d'Obsidian_Rs (100% Rust)

use obsidian_rs::cdp::target::CdpTarget;
use obsidian_rs::config::Config;
use obsidian_rs::injector::compiler;
use obsidian_rs::installer::detect;

#[test]
fn test_config_defaults_are_french() {
    let cfg = Config::default();
    assert_eq!(cfg.lang, "fr");
    assert_eq!(cfg.language, "fr");
    assert_eq!(cfg.palette, "obsidian");
    assert_eq!(cfg.motion_level, "ultra");
}

#[test]
fn test_config_serialization() {
    let mut cfg = Config::default();
    cfg.set_property("palette", "warm").unwrap();
    assert_eq!(cfg.palette, "warm");

    let json = cfg.to_compact_json();
    assert!(json.contains("\"lang\":\"fr\""));
    assert!(json.contains("\"palette\":\"warm\""));
}

#[test]
fn test_injection_expression_generation() {
    let cfg = Config::default();
    let expr = compiler::build_inject_expression(&cfg, Some("body { background: black; }"));
    assert!(expr.contains("ag-prism-css"));
    assert!(expr.contains("Edmond NOUMEGNI"));
    assert!(expr.contains("\"lang\":\"fr\""));
}

#[test]
fn test_removal_expression() {
    let expr = compiler::build_remove_expression();
    assert!(expr.contains("ag-prism-css"));
    assert!(expr.contains("desactive"));
}

#[test]
fn test_target_eligibility() {
    let target_eligible = CdpTarget {
        id: "target-1".to_string(),
        title: "Google Antigravity Workbench".to_string(),
        target_type: "page".to_string(),
        url: "vscode-file://vscode-app/workbench.html".to_string(),
        web_socket_debugger_url: Some("ws://127.0.0.1:9222/devtools/page/target-1".to_string()),
    };
    assert!(target_eligible.is_eligible());

    let target_devtools = CdpTarget {
        id: "target-2".to_string(),
        title: "DevTools".to_string(),
        target_type: "page".to_string(),
        url: "devtools://devtools/bundled/inspector.html".to_string(),
        web_socket_debugger_url: Some("ws://127.0.0.1:9222/devtools/page/target-2".to_string()),
    };
    assert!(!target_devtools.is_eligible());

    let target_worker = CdpTarget {
        id: "target-3".to_string(),
        title: "Service Worker".to_string(),
        target_type: "service_worker".to_string(),
        url: "file:///worker.js".to_string(),
        web_socket_debugger_url: Some("ws://127.0.0.1:9222/devtools/page/target-3".to_string()),
    };
    assert!(!target_worker.is_eligible());
}

#[test]
fn test_antigravity_detection() {
    let detection = detect::detect_antigravity();
    // Sur cette machine, Antigravity.exe est bien présent
    assert!(detection.exe_path.is_some());
    let path = detection.exe_path.unwrap();
    assert!(path.to_string_lossy().contains("Antigravity"));
}

#[test]
fn test_asar_strip_and_patch() {
    use obsidian_rs::installer::asar;
    let original = "console.log('original preload');";
    let patched = asar::generate_patched_preload(original, "console.log('prism');");
    assert!(patched.contains(asar::PATCH_START_MARKER));
    assert!(patched.contains(asar::PATCH_END_MARKER));
    assert!(patched.contains("console.log('prism');"));

    let cleaned = asar::strip_marker_block(&patched, asar::PATCH_START_MARKER, asar::PATCH_END_MARKER);
    assert!(!cleaned.contains(asar::PATCH_START_MARKER));
    assert!(cleaned.contains("console.log('original preload');"));
}

#[test]
fn test_vscode_extensions_finding() {
    use obsidian_rs::installer::vscode;
    let exts = vscode::find_antigravity_vscode_extensions();
    // Sur cette machine, google.google-antigravity-1.13.0 est présente
    assert!(!exts.is_empty());
    assert!(exts.iter().any(|e| e.name.contains("antigravity")));
}
