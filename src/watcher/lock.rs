// Gestion adaptative du fichier de verrouillage (.watch.pid) pour le démon de surveillance

use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

pub const PID_FILE_NAME: &str = ".watch.pid";

/// Résout l'emplacement du fichier de verrouillage de manière robuste sur n'importe quel PC.
pub fn get_resolved_pid_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.join(PID_FILE_NAME);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".gemini").join("config").join("plugins").join("obsidian-prism");
        let _ = std::fs::create_dir_all(&p);
        return p.join(PID_FILE_NAME);
    }
    PathBuf::from(PID_FILE_NAME)
}

pub struct PidLock {
    path: PathBuf,
}

impl PidLock {
    /// Tente d'acquérir le verrou du démon. Renvoie une erreur si une autre instance est active.
    pub fn acquire<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref().to_path_buf();
        if p.exists() {
            if let Ok(content) = fs::read_to_string(&p) {
                if let Ok(pid_num) = content.trim().parse::<u32>() {
                    let mut s = System::new_with_specifics(
                        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
                    );
                    s.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid_num)]), true);
                    if s.process(Pid::from_u32(pid_num)).is_some() {
                        return Err(format!(
                            "Un démon de surveillance est déjà actif avec le PID {}. Veuillez l'arrêter avant d'en lancer un nouveau.",
                            pid_num
                        ));
                    }
                }
            }
            // Fichier résiduel d'une session précédente terminée
            let _ = fs::remove_file(&p);
        }

        let my_pid = std::process::id();
        fs::write(&p, my_pid.to_string())
            .map_err(|e| format!("Impossible d'écrire le fichier de verrouillage '{:?}': {}", p, e))?;

        Ok(Self { path: p })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
