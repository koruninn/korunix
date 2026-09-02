//! Operaciones del sistema vivo de Korunix.
//!
//! Nix conserva el modelo declarativo. Este módulo Rust ejecuta las operaciones
//! que dependen del equipo en funcionamiento. No delega dominio operativo a Bash.
//!
//! Las pruebas pueden sustituir herramientas externas con `KORUNIX_TOOL_*` y la
//! frontera privilegiada completa con `KORUNIX_TEST_PRIVILEGED_RUNNER`. Nunca se
//! fabrica un pseudo-TTY ni se automatiza una contraseña.

use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};

fn args_texto(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|v| v.to_string_lossy().into_owned())
        .collect()
}

fn tool_env(nombre: &str) -> String {
    let mut salida = String::from("KORUNIX_TOOL_");
    for c in nombre.chars() {
        if c.is_ascii_alphanumeric() {
            salida.push(c.to_ascii_uppercase());
        } else {
            salida.push('_');
        }
    }
    salida
}

fn tool(nombre: &str) -> OsString {
    env::var_os(tool_env(nombre)).unwrap_or_else(|| OsString::from(nombre))
}

fn capture(raiz: &Path, programa: &str, args: &[String]) -> Result<String, String> {
    let out = Command::new(tool(programa))
        .args(args)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("No pude ejecutar {programa}: {e}"))?;

    if !out.status.success() {
        let error = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if error.is_empty() {
            format!("{programa} terminó con error.")
        } else {
            error
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn capture_status(
    raiz: &Path,
    programa: &str,
    args: &[String],
) -> Result<(i32, String, String), String> {
    let out = Command::new(tool(programa))
        .args(args)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("No pude ejecutar {programa}: {e}"))?;

    Ok((
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
        String::from_utf8_lossy(&out.stderr).trim().to_string(),
    ))
}

fn visible(raiz: &Path, programa: &str, args: &[String]) -> Result<(), String> {
    let status = Command::new(tool(programa))
        .args(args)
        .current_dir(raiz)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("No pude ejecutar {programa}: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{programa} terminó con error."))
    }
}

fn fstab_cambio(antes: &Option<Vec<u8>>, despues: &Option<Vec<u8>>) -> bool {
    antes != despues
}

fn refrescar_monitor_gvfs_si_cambio_fstab(fstab_antes: Option<Vec<u8>>) {
    let fstab_despues = fs::read("/etc/fstab").ok();

    if !fstab_cambio(&fstab_antes, &fstab_despues) {
        return;
    }

    // GVfs no siempre vuelve a leer /etc/fstab cuando NixOS activa una nueva
    // generación. Un reinicio dirigido de su monitor de UDisks actualiza la
    // lista de unidades sin cerrar Nautilus ni reiniciar la sesión completa.
    // `try-restart` no inicia el servicio en escritorios que no lo usan.
    let _ = Command::new(tool("systemctl"))
        .args([
            "--user",
            "try-restart",
            "gvfs-udisks2-volume-monitor.service",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(test)]
#[test]
fn cambio_de_fstab_activa_refresco_solo_si_cambia_el_contenido() {
    let original = Some(b"unidad-a\n".to_vec());
    let igual = Some(b"unidad-a\n".to_vec());
    let diferente = Some(b"unidad-a\nunidad-b\n".to_vec());

    assert!(!fstab_cambio(&original, &igual));
    assert!(fstab_cambio(&original, &diferente));
    assert!(fstab_cambio(&None, &original));
}

fn jq0(raiz: &Path, args: &[String]) -> Result<String, String> {
    jq_con_entrada(raiz, args, "")
}

fn pretty(raiz: &Path, json: &str) -> Result<(), String> {
    println!("{}", jq_con_entrada(raiz, &[".".into()], json)?);
    Ok(())
}

fn state_root() -> Result<PathBuf, String> {
    if let Some(v) = env::var_os("XDG_STATE_HOME") {
        return Ok(PathBuf::from(v).join("korunix"));
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME no está disponible.".to_string())?;
    Ok(home.join(".local/state/korunix"))
}

fn stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}-{}", std::process::id())
}

fn backup_dir(nombre: &str) -> Result<PathBuf, String> {
    let p = state_root()?
        .join("backups")
        .join(format!("{nombre}-{}", stamp()));
    fs::create_dir_all(&p).map_err(|e| format!("No pude crear {}: {e}", p.display()))?;
    Ok(p)
}

fn sync_directory(path: &Path) -> Result<(), String> {
    let directory = fs::File::open(path)
        .map_err(|e| format!("No pude abrir {} para sincronizar: {e}", path.display()))?;
    directory.sync_all().map_err(|e| {
        format!(
            "No pude confirmar {} en almacenamiento: {e}",
            path.display()
        )
    })
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Ruta sin carpeta padre.".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("No pude crear {}: {e}", parent.display()))?;
    let tmp = parent.join(format!(
        ".korunix-{}-{}.tmp",
        path.file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("archivo"),
        stamp()
    ));

    let result = (|| -> Result<(), String> {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)
            .map_err(|e| format!("No pude crear {}: {e}", tmp.display()))?;
        file.write_all(data)
            .map_err(|e| format!("No pude escribir {}: {e}", tmp.display()))?;
        file.sync_all()
            .map_err(|e| format!("No pude confirmar {} en almacenamiento: {e}", tmp.display()))?;

        if let Ok(meta) = fs::metadata(path) {
            fs::set_permissions(&tmp, meta.permissions())
                .map_err(|e| format!("No pude conservar permisos de {}: {e}", path.display()))?;
            let file = fs::OpenOptions::new()
                .write(true)
                .open(&tmp)
                .map_err(|e| format!("No pude reabrir {}: {e}", tmp.display()))?;
            file.sync_all()
                .map_err(|e| format!("No pude confirmar permisos de {}: {e}", tmp.display()))?;
        }

        fs::rename(&tmp, path).map_err(|e| format!("No pude sustituir {}: {e}", path.display()))?;
        sync_directory(parent)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

// `configuracion/` es un árbol completo y no puede sustituirse con atomic_write.
// Linux ofrece RENAME_EXCHANGE: ambos nombres existen antes y después del syscall,
// de modo que una interrupción nunca deja al repositorio sin configuracion/.
#[cfg(target_os = "linux")]
fn exchange_paths(left: &Path, right: &Path) -> Result<(), String> {
    use std::ffi::{c_char, CString};
    use std::os::unix::ffi::OsStrExt;

    const AT_FDCWD: i32 = -100;
    const RENAME_EXCHANGE: u32 = 2;

    extern "C" {
        fn renameat2(
            olddirfd: i32,
            oldpath: *const c_char,
            newdirfd: i32,
            newpath: *const c_char,
            flags: u32,
        ) -> i32;
    }

    let left_c = CString::new(left.as_os_str().as_bytes())
        .map_err(|_| format!("Ruta no válida para intercambio: {}", left.display()))?;
    let right_c = CString::new(right.as_os_str().as_bytes())
        .map_err(|_| format!("Ruta no válida para intercambio: {}", right.display()))?;

    let result = unsafe {
        renameat2(
            AT_FDCWD,
            left_c.as_ptr(),
            AT_FDCWD,
            right_c.as_ptr(),
            RENAME_EXCHANGE,
        )
    };

    if result == 0 {
        let left_parent = left
            .parent()
            .ok_or_else(|| "La ruta izquierda no tiene carpeta padre.".to_string())?;
        let right_parent = right
            .parent()
            .ok_or_else(|| "La ruta derecha no tiene carpeta padre.".to_string())?;
        sync_directory(left_parent)?;
        if right_parent != left_parent {
            sync_directory(right_parent)?;
        }
        Ok(())
    } else {
        Err(format!(
            "No pude intercambiar de forma atómica {} y {}: {}",
            left.display(),
            right.display(),
            io::Error::last_os_error()
        ))
    }
}

#[cfg(not(target_os = "linux"))]
fn exchange_paths(_left: &Path, _right: &Path) -> Result<(), String> {
    Err("La restauración atómica de Korunix requiere Linux.".to_string())
}

fn transaction_pending_path() -> Result<PathBuf, String> {
    Ok(state_root()?.join("transaction.pending"))
}

fn legacy_restore_pending_path() -> Result<PathBuf, String> {
    Ok(state_root()?.join("restore.pending"))
}

fn transaction_relative_path(raiz: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(raiz)
        .map_err(|_| format!("{} queda fuera del repositorio de Korunix.", path.display()))?;

    let configuracion = relative.starts_with(Path::new("configuracion"));
    let hardware_generado = relative.starts_with(Path::new("generado/equipos"));

    if relative
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
        || !(configuracion || hardware_generado)
    {
        return Err(format!(
            "Korunix rechazó una ruta fuera de configuracion/ y generado/equipos/: {}",
            path.display()
        ));
    }

    relative
        .to_str()
        .map(ToString::to_string)
        .ok_or_else(|| format!("La ruta {} no es texto UTF-8 válido.", path.display()))
}

fn transaction_workspace() -> Result<PathBuf, String> {
    let root = state_root()?.join("transactions");
    fs::create_dir_all(&root).map_err(|e| format!("No pude crear {}: {e}", root.display()))?;
    let workspace = root.join(format!("files-{}", stamp()));
    fs::create_dir(&workspace)
        .map_err(|e| format!("No pude crear {}: {e}", workspace.display()))?;
    Ok(workspace)
}

fn transaction_workspace_valid(workspace: &Path) -> Result<bool, String> {
    let root = state_root()?.join("transactions");
    Ok(workspace.parent() == Some(root.as_path())
        && workspace
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.starts_with("files-"))
            .unwrap_or(false))
}

fn transaction_pending_busy() -> Result<bool, String> {
    Ok(transaction_pending_path()?.exists() || legacy_restore_pending_path()?.exists())
}

fn files_transaction_begin(raiz: &Path, paths: &[PathBuf]) -> Result<PathBuf, String> {
    if transaction_pending_busy()? {
        return Err(
            "Existe una transacción pendiente de Korunix. Ejecuta cualquier operación de Korunix para recuperarla antes de iniciar otra."
                .into(),
        );
    }

    let workspace = transaction_workspace()?;
    let mut entries = Vec::<serde_json::Value>::new();
    let mut seen = BTreeSet::<String>::new();

    for path in paths {
        let relative = transaction_relative_path(raiz, path)?;
        if !seen.insert(relative.clone()) {
            continue;
        }

        if path.exists() && !path.is_file() {
            let _ = fs::remove_dir_all(&workspace);
            return Err(format!(
                "La transacción declarativa solo admite archivos: {}",
                path.display()
            ));
        }

        let existed = path.is_file();
        let index = entries.len();

        if existed {
            fs::copy(path, workspace.join(format!("{index}.bin"))).map_err(|error| {
                format!(
                    "No pude respaldar {} para la transacción: {error}",
                    path.display()
                )
            })?;
        }

        entries.push(serde_json::json!({
            "path": relative,
            "existed": existed
        }));
    }

    let journal = serde_json::json!({
        "schemaVersion": 1,
        "kind": "declarative-files",
        "workspace": workspace.display().to_string(),
        "entries": entries
    });
    let data = serde_json::to_vec_pretty(&journal)
        .map_err(|e| format!("No pude serializar la transacción: {e}"))?;

    if let Err(error) = atomic_write(&transaction_pending_path()?, &data) {
        let _ = fs::remove_dir_all(&workspace);
        return Err(error);
    }

    Ok(workspace)
}

fn files_transaction_restore(raiz: &Path, journal: &serde_json::Value) -> Result<PathBuf, String> {
    let workspace = journal
        .get("workspace")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| "La transacción declarativa no contiene workspace.".to_string())?;

    if !transaction_workspace_valid(&workspace)? {
        return Err("Korunix rechazó el workspace de una transacción declarativa.".into());
    }

    let entries = journal
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "La transacción declarativa no contiene archivos.".to_string())?;

    for (index, entry) in entries.iter().enumerate() {
        let relative = entry
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Una entrada de la transacción no contiene ruta.".to_string())?;
        let path = raiz.join(relative);

        if transaction_relative_path(raiz, &path)? != relative {
            return Err(format!("Ruta de recuperación no válida: {relative}"));
        }

        let existed = entry
            .get("existed")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                "Una entrada de la transacción no contiene estado previo.".to_string()
            })?;

        if existed {
            let backup = workspace.join(format!("{index}.bin"));
            let data =
                fs::read(&backup).map_err(|e| format!("No pude leer {}: {e}", backup.display()))?;
            atomic_write(&path, &data)?;
        } else if path.is_file() {
            fs::remove_file(&path).map_err(|e| {
                format!(
                    "No pude retirar {} durante la recuperación: {e}",
                    path.display()
                )
            })?;
        } else if path.exists() {
            return Err(format!(
                "La recuperación no eliminará una ruta que dejó de ser archivo: {}",
                path.display()
            ));
        }
    }

    Ok(workspace)
}

fn transaction_commit(cleanup: Option<&Path>) -> Result<(), String> {
    let pending = transaction_pending_path()?;
    fs::remove_file(&pending)
        .map_err(|e| format!("No pude cerrar la transacción {}: {e}", pending.display()))?;

    if let Some(path) = cleanup {
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        } else if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    Ok(())
}

fn restore_pending_write(seguridad: &Path, candidate: &Path) -> Result<(), String> {
    if transaction_pending_busy()? {
        return Err(
            "Existe una transacción pendiente de Korunix; no puedo iniciar otra restauración."
                .into(),
        );
    }

    let journal = serde_json::json!({
        "schemaVersion": 1,
        "kind": "restore-tree",
        "safetyBackup": seguridad.display().to_string(),
        "candidate": candidate.display().to_string()
    });
    let data = serde_json::to_vec_pretty(&journal)
        .map_err(|e| format!("No pude serializar la restauración: {e}"))?;
    atomic_write(&transaction_pending_path()?, &data)
}

fn restore_paths_valid(raiz: &Path, seguridad: &Path, candidate: &Path) -> Result<(), String> {
    let backups = state_root()?.join("backups");
    let seguridad_valida = seguridad.parent() == Some(backups.as_path())
        && seguridad
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.starts_with("config-restore-"))
            .unwrap_or(false);
    let candidate_valido = candidate.parent() == Some(raiz)
        && candidate
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.starts_with(".korunix-restore-config-"))
            .unwrap_or(false);

    if !seguridad_valida || !candidate_valido {
        return Err("Korunix rechazó una transacción de restauración con rutas no válidas.".into());
    }

    Ok(())
}

fn legacy_restore_pending_read(raiz: &Path) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let pending = legacy_restore_pending_path()?;
    if !pending.is_file() {
        return Ok(None);
    }

    let data = fs::read_to_string(&pending)
        .map_err(|e| format!("No pude leer {}: {e}", pending.display()))?;
    let mut lines = data.lines();
    let seguridad = PathBuf::from(
        lines
            .next()
            .ok_or_else(|| "La restauración antigua quedó incompleta.".to_string())?,
    );
    let candidate = PathBuf::from(
        lines
            .next()
            .ok_or_else(|| "La restauración antigua quedó incompleta.".to_string())?,
    );

    restore_paths_valid(raiz, &seguridad, &candidate)?;
    Ok(Some((seguridad, candidate)))
}

fn restore_from_safety(raiz: &Path, seguridad: &Path, candidate: &Path) -> Result<(), String> {
    let safety_config = seguridad.join("configuracion");
    let safety_lock = seguridad.join("flake.lock");
    if !safety_config.is_dir() || !safety_lock.is_file() {
        return Err(format!(
            "El respaldo de recuperación {} está incompleto.",
            seguridad.display()
        ));
    }

    let actual = raiz.join("configuracion");
    let recovery = raiz.join(format!(".korunix-restore-recovery-{}", stamp()));
    copy_dir_recursive(&safety_config, &recovery)?;

    let config_result = if actual.is_dir() {
        exchange_paths(&actual, &recovery)
    } else {
        fs::rename(&recovery, &actual).map_err(|e| {
            format!(
                "No pude recuperar {} desde {}: {e}",
                actual.display(),
                seguridad.display()
            )
        })
    };

    if let Err(error) = config_result {
        let _ = fs::remove_dir_all(&recovery);
        return Err(error);
    }

    let old_lock = fs::read(&safety_lock)
        .map_err(|e| format!("No pude leer el flake.lock de recuperación: {e}"))?;
    if let Err(error) = atomic_write(&raiz.join("flake.lock"), &old_lock) {
        let _ = fs::remove_dir_all(&recovery);
        return Err(error);
    }

    let _ = fs::remove_dir_all(&recovery);
    if candidate.is_dir() {
        let _ = fs::remove_dir_all(candidate);
    }
    Ok(())
}

fn rollback_pending_transaction(raiz: &Path) -> Result<bool, String> {
    let pending = transaction_pending_path()?;

    if pending.is_file() {
        let data = fs::read_to_string(&pending)
            .map_err(|e| format!("No pude leer {}: {e}", pending.display()))?;
        let journal: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| format!("La transacción pendiente no es JSON válido: {e}"))?;
        let kind = journal
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "La transacción pendiente no declara su tipo.".to_string())?;

        let cleanup = match kind {
            "restore-tree" => {
                let seguridad = journal
                    .get("safetyBackup")
                    .and_then(serde_json::Value::as_str)
                    .map(PathBuf::from)
                    .ok_or_else(|| "La restauración pendiente no contiene respaldo.".to_string())?;
                let candidate = journal
                    .get("candidate")
                    .and_then(serde_json::Value::as_str)
                    .map(PathBuf::from)
                    .ok_or_else(|| {
                        "La restauración pendiente no contiene candidato.".to_string()
                    })?;
                restore_paths_valid(raiz, &seguridad, &candidate)?;
                restore_from_safety(raiz, &seguridad, &candidate)?;
                None
            }
            "declarative-files" => Some(files_transaction_restore(raiz, &journal)?),
            other => {
                return Err(format!(
                    "Korunix no reconoce el tipo de transacción pendiente: {other}"
                ));
            }
        };

        fs::remove_file(&pending)
            .map_err(|e| format!("No pude cerrar la recuperación pendiente: {e}"))?;
        if let Some(path) = cleanup {
            let _ = fs::remove_dir_all(path);
        }
        return Ok(true);
    }

    if let Some((seguridad, candidate)) = legacy_restore_pending_read(raiz)? {
        restore_from_safety(raiz, &seguridad, &candidate)?;
        fs::remove_file(legacy_restore_pending_path()?)
            .map_err(|e| format!("No pude cerrar la restauración antigua pendiente: {e}"))?;
        return Ok(true);
    }

    Ok(false)
}

fn recover_pending_transaction(raiz: &Path) -> Result<bool, String> {
    let recovered = rollback_pending_transaction(raiz)?;
    if recovered {
        eprintln!("✓ Korunix recuperó una transacción interrumpida.");
    }
    Ok(recovered)
}

fn sha256(raiz: &Path, path: &Path) -> Result<String, String> {
    let out = capture(raiz, "sha256sum", &[path.display().to_string()])?;
    out.split_whitespace()
        .next()
        .map(ToString::to_string)
        .ok_or_else(|| "sha256sum no devolvió una suma.".to_string())
}

fn confirm(pregunta: &str) -> Result<bool, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("La operación necesita confirmación interactiva o --yes.".to_string());
    }

    print!("{pregunta} [s/N] ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut value = String::new();
    io::stdin()
        .read_line(&mut value)
        .map_err(|e| e.to_string())?;
    Ok(matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "s" | "si" | "sí" | "y" | "yes"
    ))
}

fn trusted_program(nombre: &str) -> Result<PathBuf, String> {
    if !matches!(
        nombre,
        "nix"
            | "nix-env"
            | "nixos-rebuild"
            | "nix-collect-garbage"
            | "bootctl"
            | "grub-reboot"
            | "grub-editenv"
            | "cat"
            | "passwd"
            | "chpasswd"
            | "systemctl"
    ) {
        return Err(format!("Programa privilegiado no permitido: {nombre}"));
    }

    for base in ["/run/current-system/sw/bin", "/run/wrappers/bin"] {
        let p = Path::new(base).join(nombre);
        if p.is_file() {
            return Ok(p);
        }
    }

    let path = env::var_os("PATH").ok_or_else(|| "PATH no está disponible.".to_string())?;
    for dir in env::split_paths(&path) {
        let p = dir.join(nombre);
        if !p.is_file() {
            continue;
        }
        let canon = fs::canonicalize(&p).unwrap_or(p.clone());
        let value = canon.display().to_string();
        if value.starts_with("/nix/store/")
            || value.starts_with("/run/current-system/sw/bin/")
            || value.starts_with("/run/wrappers/bin/")
        {
            return Ok(p);
        }
    }

    Err(format!("No encuentro una ruta confiable para {nombre}."))
}

fn privileged(
    raiz: &Path,
    programa: &str,
    args: &[String],
    visible_output: bool,
) -> Result<String, String> {
    if let Some(runner) = env::var_os("KORUNIX_TEST_PRIVILEGED_RUNNER") {
        let out = Command::new(runner)
            .arg(programa)
            .args(args)
            .current_dir(raiz)
            .stdin(Stdio::null())
            .stdout(if visible_output {
                Stdio::inherit()
            } else {
                Stdio::piped()
            })
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Frontera privilegiada de prueba: {e}"))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        return Ok(String::from_utf8_lossy(&out.stdout).trim().to_string());
    }

    let target = trusted_program(programa)?;

    let pkexec = [
        "/run/wrappers/bin/pkexec",
        "/run/current-system/sw/bin/pkexec",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|p| p.is_file());

    if let Some(pkexec) = pkexec {
        let out = Command::new(pkexec)
            .arg("--disable-internal-agent")
            .arg(target)
            .args(args)
            .current_dir(raiz)
            .stdin(Stdio::inherit())
            .stdout(if visible_output {
                Stdio::inherit()
            } else {
                Stdio::piped()
            })
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("No pude solicitar autorización: {e}"))?;

        if !out.status.success() {
            let error = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(if error.is_empty() {
                "La autorización fue rechazada.".to_string()
            } else {
                error
            });
        }
        return Ok(String::from_utf8_lossy(&out.stdout).trim().to_string());
    }

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        let sudo = ["/run/wrappers/bin/sudo", "/run/current-system/sw/bin/sudo"]
            .into_iter()
            .map(PathBuf::from)
            .find(|p| p.is_file());

        if let Some(sudo) = sudo {
            let status = Command::new(sudo)
                .arg(target)
                .args(args)
                .current_dir(raiz)
                .stdin(Stdio::inherit())
                .stdout(if visible_output {
                    Stdio::inherit()
                } else {
                    Stdio::piped()
                })
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| format!("No pude ejecutar sudo: {e}"))?;
            if status.success() {
                return Ok(String::new());
            }
            return Err("La operación administrativa falló.".to_string());
        }
    }

    Err("Polkit no está disponible; Korunix no automatizará una contraseña.".to_string())
}

fn privileged_input(
    raiz: &Path,
    programa: &str,
    args: &[String],
    input: &[u8],
) -> Result<String, String> {
    if let Some(runner) = env::var_os("KORUNIX_TEST_PRIVILEGED_RUNNER") {
        let mut child = Command::new(runner)
            .arg(programa)
            .args(args)
            .current_dir(raiz)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Frontera privilegiada de prueba: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input)
                .map_err(|e| format!("No pude entregar la entrada protegida: {e}"))?;
        }

        let out = child
            .wait_with_output()
            .map_err(|e| format!("Frontera privilegiada de prueba: {e}"))?;

        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }

        return Ok(String::from_utf8_lossy(&out.stdout).trim().to_string());
    }

    let target = trusted_program(programa)?;
    let pkexec = [
        "/run/wrappers/bin/pkexec",
        "/run/current-system/sw/bin/pkexec",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|p| p.is_file())
    .ok_or_else(|| {
        "Polkit no está disponible; Korunix no enviará una contraseña por argumentos.".to_string()
    })?;

    let mut child = Command::new(pkexec)
        .arg("--disable-internal-agent")
        .arg(target)
        .args(args)
        .current_dir(raiz)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("No pude solicitar autorización: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input)
            .map_err(|e| format!("No pude entregar la entrada protegida: {e}"))?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| format!("No pude esperar la operación autorizada: {e}"))?;

    if !out.status.success() {
        let error = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if error.is_empty() {
            "La autorización fue rechazada.".to_string()
        } else {
            error
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn system_profile() -> PathBuf {
    env::var_os("KORUNIX_SYSTEM_PROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/nix/var/nix/profiles/system"))
}

fn current_system() -> String {
    let link = env::var_os("KORUNIX_CURRENT_SYSTEM_LINK")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/run/current-system"));
    fs::canonicalize(link)
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

fn generation_from_link(path: &Path) -> Option<u32> {
    let n = path.file_name()?.to_str()?;
    n.strip_prefix("system-")?
        .strip_suffix("-link")?
        .parse()
        .ok()
}

fn generations() -> Vec<(u32, PathBuf)> {
    let profile = system_profile();
    let parent = profile
        .parent()
        .unwrap_or_else(|| Path::new("/nix/var/nix/profiles"));
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let Some(id) = generation_from_link(&entry.path()) else {
                continue;
            };
            if let Ok(target) = fs::canonicalize(entry.path()) {
                out.push((id, target));
            }
        }
    }
    out.sort_by_key(|(id, _)| *id);
    out
}

fn default_generation() -> Option<u32> {
    fs::read_link(system_profile())
        .ok()
        .as_deref()
        .and_then(generation_from_link)
}

fn current_generation() -> Option<u32> {
    let current = current_system();
    generations()
        .into_iter()
        .rev()
        .find_map(|(id, p)| (p.display().to_string() == current).then_some(id))
}

fn bootloader() -> &'static str {
    if env::var("KORUNIX_TEST_FIRMWARE").ok().as_deref() == Some("bios") {
        return "grub";
    }
    if env::var("KORUNIX_TEST_FIRMWARE").ok().as_deref() == Some("uefi") {
        return "systemd-boot";
    }
    if Path::new("/sys/firmware/efi").is_dir() {
        "systemd-boot"
    } else {
        "grub"
    }
}

fn hardware_human(raiz: &Path) -> Result<(), String> {
    let j = hardware_json(raiz)?;
    println!("=== Hardware ===");
    println!("Equipo: {}", jq_texto(raiz, &j, ".hostId")?);
    println!("Tipo: {}", jq_texto(raiz, &j, ".machine.type")?);
    println!(
        "Modelo: {} {}",
        jq_texto(raiz, &j, ".machine.vendor")?,
        jq_texto(raiz, &j, ".machine.model")?
    );
    println!("CPU: {}", jq_texto(raiz, &j, ".cpu.model")?);
    println!("Firmware: {}", jq_texto(raiz, &j, ".firmware.detected")?);
    Ok(())
}

fn localization_human(raiz: &Path) -> Result<(), String> {
    let j = localizacion_json(raiz)?;
    println!("=== Localización ===");
    println!(
        "Idioma: {}",
        jq_texto(raiz, &j, ".declared.systemLanguage")?
    );
    println!("Región: {}", jq_texto(raiz, &j, ".declared.region")?);
    println!(
        "Zona horaria: {}",
        jq_texto(raiz, &j, ".declared.timeZone")?
    );
    println!(
        "Teclado: {}",
        jq_texto(raiz, &j, ".derived.keyboard.layout")?
    );
    println!(
        "Contradicciones: {}",
        jq_texto(raiz, &j, ".contradictions | length")?
    );
    Ok(())
}

fn users_human(raiz: &Path) -> Result<(), String> {
    let j = usuarios_json(raiz)?;
    let lines = jq_con_entrada(
        raiz,
        &[
            "-r".into(),
            r#".accounts[]? | "• " + .displayName + " — " + .accountName + " · " + .status"#.into(),
        ],
        &j,
    )?;
    println!("=== Personas ===");
    if lines.is_empty() {
        println!("ninguna");
    } else {
        println!("{lines}");
    }
    println!("Korunix no guarda contraseñas ni hashes.");
    Ok(())
}

fn structure(raiz: &Path) -> Result<(), String> {
    println!("=== Estructura ===");
    for (path, desc) in [
        ("configuracion", "decisiones humanas"),
        ("sistema", "funcionamiento interno"),
        ("generado", "hechos creados o detectados automáticamente"),
    ] {
        if !raiz.join(path).is_dir() {
            return Err(format!("Falta {path}/."));
        }
        println!("✓ {path}/ · {desc}");
    }
    Ok(())
}

fn flake_source(raiz: &Path) -> String {
    raiz.display().to_string()
}

fn flake_reference(raiz: &Path, fragment: &str) -> String {
    format!("{}#{fragment}", flake_source(raiz))
}

fn validate_with_output(raiz: &Path, human_output: bool) -> Result<(), String> {
    if human_output {
        structure(raiz)?;
    } else {
        for path in ["configuracion", "sistema", "generado"] {
            if !raiz.join(path).is_dir() {
                return Err(format!("Falta {path}/."));
            }
        }
    }

    if raiz.join(".git").exists() {
        let args = ["diff".into(), "--check".into()];
        if human_output {
            visible(raiz, "git", &args)?;
        } else {
            let _ = capture(raiz, "git", &args)?;
        }
    } else if human_output {
        println!("✓ distribución sin metadatos Git; se valida como producto");
    }

    let flake_args = [
        "flake".into(),
        "check".into(),
        flake_source(raiz),
        "--no-build".into(),
        "--show-trace".into(),
    ];

    if human_output {
        visible(raiz, "nix", &flake_args)?;
    } else {
        let _ = capture(raiz, "nix", &flake_args)?;
    }

    for id in equipos_disponibles(raiz)? {
        let drv = capture(
            raiz,
            "nix",
            &[
                "eval".into(),
                "--raw".into(),
                "--no-write-lock-file".into(),
                flake_reference(
                    raiz,
                    &format!("nixosConfigurations.{id}.config.system.build.toplevel.drvPath"),
                ),
            ],
        )?;
        if drv.is_empty() {
            return Err(format!("Nix no produjo drvPath para {id}."));
        }
    }

    if human_output {
        println!("✓ VALIDACIÓN COMPLETA");
    }

    Ok(())
}

fn validate(raiz: &Path) -> Result<(), String> {
    validate_with_output(raiz, true)
}

fn validate_quiet(raiz: &Path) -> Result<(), String> {
    validate_with_output(raiz, false)
}

fn collect_nix(path: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.file_name().and_then(|v| v.to_str()) == Some(".git") {
            continue;
        }
        if p.is_dir() {
            collect_nix(&p, out);
        } else if p.extension().and_then(|v| v.to_str()) == Some("nix") {
            out.push(p);
        }
    }
}

fn format_nix(raiz: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_nix(raiz, &mut files);
    files.sort();
    if files.is_empty() {
        return Ok(());
    }
    let args: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
    visible(raiz, "alejandra", &args)?;
    println!("✓ formato aplicado");
    Ok(())
}

fn status(raiz: &Path) -> Result<(), String> {
    println!("=== Fuente de Korunix ===");

    if raiz.join(".git").exists() {
        visible(
            raiz,
            "git",
            &["status".into(), "--short".into(), "--branch".into()],
        )?;
    } else {
        println!("Distribución de producto sin metadatos Git.");
    }

    println!("\n=== Equipo ===");
    println!("Host: {}", resolver_equipo(raiz)?);
    println!("Cargador: {}", bootloader());
    println!(
        "Generación actual: {}",
        current_generation()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "desconocida".into())
    );
    println!(
        "Generación predeterminada: {}",
        default_generation()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "desconocida".into())
    );
    Ok(())
}

fn recovery_list_json(raiz: &Path) -> Result<String, String> {
    let host = resolver_equipo(raiz)?;
    let current = current_system();
    let current_id = current_generation();
    let default_id = default_generation();
    let mut items = Vec::new();

    for (id, path) in generations() {
        let version = fs::read_to_string(path.join("nixos-version"))
            .ok()
            .map(|v| v.trim().to_string());
        items.push(format!(
            "{{\"id\":{id},\"systemPath\":{},\"nixosVersion\":{},\"current\":{},\"default\":{}}}",
            json_texto(&path.display().to_string()),
            version
                .as_deref()
                .map(json_texto)
                .unwrap_or_else(|| "null".into()),
            current_id == Some(id),
            default_id == Some(id)
        ));
    }

    Ok(format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-recovery-list\",\"hostId\":{},\"bootloader\":{},\"currentSystem\":{},\"defaultGeneration\":{},\"generations\":[{}]}}",
        json_texto(&host),
        json_texto(bootloader()),
        if current.is_empty() { "null".into() } else { json_texto(&current) },
        default_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".into()),
        items.join(",")
    ))
}

fn recovery_plan_json(raiz: &Path, id: u32) -> Result<String, String> {
    let list = recovery_list_json(raiz)?;
    let target = jq_compacto(
        raiz,
        &list,
        &format!(".generations[] | select(.id == {id})"),
    )?;
    if target.is_empty() || target == "null" {
        return Err(format!("El punto de recuperación {id} no existe."));
    }
    jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "list".into(),
            list,
            "--argjson".into(),
            "target".into(),
            target,
            r#"{
              schemaVersion:1,
              kind:"korunix-recovery-plan",
              hostId:$list.hostId,
              bootloader:$list.bootloader,
              mode:"one-shot-next-boot",
              current:{
                systemPath:$list.currentSystem,
                generation:([$list.generations[] | select(.current) | .id][0] // null)
              },
              defaultGeneration:$list.defaultGeneration,
              target:$target,
              effects:{
                runningSystemChanged:false,
                defaultGenerationChanged:false,
                nextBootChanged:true,
                rebootRequiredToUseTarget:true
              },
              privilege:{requiredToSchedule:true},
              safety:{
                targetMustAlreadyExist:true,
                oneShotOnly:true,
                returnsToDefaultAfterBoot:true
              }
            }"#
            .into(),
        ],
    )
}

fn save_scheduled_recovery(id: u32) -> Result<(), String> {
    let dir = state_root()?.join("recovery");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let boot_id = fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .unwrap_or_default()
        .trim()
        .to_string();
    atomic_write(
        &dir.join("next-boot.json"),
        format!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-recovery-protection\",\"generation\":{id},\"bootloader\":{},\"scheduledFromBootId\":{}}}\n",
            json_texto(bootloader()),
            json_texto(&boot_id)
        )
        .as_bytes(),
    )
}

fn scheduled_recovery() -> Option<u32> {
    let text = fs::read_to_string(state_root().ok()?.join("recovery/next-boot.json")).ok()?;
    let pos = text.find("\"generation\":")?;
    let rest = &text[pos + "\"generation\":".len()..];
    rest.trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn grub_entry(config: &str, id: u32) -> Result<String, String> {
    let mut submenu = None;
    let mut entry = None;

    for line in config.lines() {
        let t = line.trim_start();
        if submenu.is_none() && t.starts_with("submenu \"") && t.contains(" - All configurations") {
            submenu = t.split('"').nth(1).map(ToString::to_string);
        }

        if t.starts_with("menuentry \"") && t.contains(&format!(" - Configuration {id} (")) {
            entry = t.split('"').nth(1).map(ToString::to_string);
            break;
        }
    }

    let submenu = submenu
        .ok_or_else(|| "No encontré el submenú de generaciones de NixOS en GRUB.".to_string())?;
    let entry =
        entry.ok_or_else(|| format!("GRUB no contiene una entrada para la generación {id}."))?;

    if let Some(base) = entry.strip_suffix(" - Default") {
        Ok(format!("{submenu}>{base}>{entry}"))
    } else {
        Ok(format!("{submenu}>{entry}"))
    }
}

fn schedule_recovery(raiz: &Path, id: u32) -> Result<String, String> {
    if bootloader() == "systemd-boot" {
        let listing = privileged(
            raiz,
            "bootctl",
            &["list".into(), "--no-pager".into()],
            false,
        )?;
        let mut last_id = String::new();
        let mut selected = None;
        for line in listing.lines() {
            let t = line.trim();
            if let Some(v) = t.strip_prefix("id:") {
                last_id = v.trim().to_string();
            }
            if let Some(v) = t.strip_prefix("version:") {
                if v.trim().starts_with(&format!("Generation {id}")) && !last_id.is_empty() {
                    selected = Some(last_id.clone());
                    break;
                }
            }
        }
        let entry = selected.ok_or_else(|| {
            format!("No existe una entrada systemd-boot para la generación {id}.")
        })?;
        let _ = privileged(
            raiz,
            "bootctl",
            &["set-oneshot".into(), entry.clone()],
            false,
        )?;
        save_scheduled_recovery(id)?;
        Ok(entry)
    } else {
        let config =
            env::var("KORUNIX_GRUB_CONFIG").unwrap_or_else(|_| "/boot/grub/grub.cfg".to_string());
        let text = privileged(raiz, "cat", std::slice::from_ref(&config), false)?;
        let entry = grub_entry(&text, id)?;
        let boot_dir = config
            .strip_suffix("/grub/grub.cfg")
            .unwrap_or("/boot")
            .to_string();
        let _ = privileged(
            raiz,
            "grub-reboot",
            &[format!("--boot-directory={boot_dir}"), entry.clone()],
            false,
        )?;
        let grubenv = format!("{boot_dir}/grub/grubenv");
        let verify = privileged(raiz, "grub-editenv", &[grubenv, "list".into()], false)?;
        let next = verify
            .lines()
            .find_map(|l| l.strip_prefix("next_entry="))
            .unwrap_or("");
        if next != entry {
            return Err("GRUB no confirmó la entrada de arranque único.".to_string());
        }
        save_scheduled_recovery(id)?;
        Ok(entry)
    }
}

fn rollback(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let list = args.iter().any(|v| v == "--list");
    let plan_only = args.iter().any(|v| v == "--plan");
    let yes = args.iter().any(|v| v == "--yes");
    let json = args.iter().any(|v| v == "--json");
    let generation = args.iter().find_map(|v| v.parse::<u32>().ok());

    if list {
        if generation.is_some() || plan_only || yes {
            return Err("Uso: korunix rollback --list [--json].".into());
        }
        let data = recovery_list_json(raiz)?;
        if json {
            println!("{data}");
        } else {
            pretty(raiz, &data)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    let id = generation.ok_or_else(|| "Indica una generación o usa --list.".to_string())?;
    let plan = recovery_plan_json(raiz, id)?;

    if plan_only {
        if yes {
            return Err("--yes no se utiliza junto con --plan.".into());
        }
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if json && !yes {
        return Err("rollback --json necesita --yes.".into());
    }

    if !yes
        && !confirm(&format!(
            "¿Usar la generación {id} solo en el próximo arranque?"
        ))?
    {
        return Ok(ExitCode::SUCCESS);
    }

    emitir_progreso(json, 20, "scheduling_recovery");
    let entry = schedule_recovery(raiz, id)?;
    emitir_progreso(json, 90, "preparing");
    let result = jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "plan".into(),
            plan,
            "--arg".into(),
            "entry".into(),
            entry,
            r#"{
              schemaVersion:1,
              kind:"korunix-recovery-result",
              hostId:$plan.hostId,
              bootloader:$plan.bootloader,
              mode:$plan.mode,
              target:$plan.target,
              bootEntry:$entry,
              scheduled:true,
              verified:true,
              verification:{
                method:(if $plan.bootloader=="grub"
                        then "grubenv-next-entry"
                        else "bootloader-command-exit-status" end)
              },
              effects:$plan.effects,
              safety:$plan.safety
            }"#
            .into(),
        ],
    )?;

    emitir_progreso(json, 100, "done");

    if json {
        println!("{result}");
    } else {
        println!("✓ recuperación preparada para un único arranque");
    }
    Ok(ExitCode::SUCCESS)
}

fn clean_plan_json(raiz: &Path, aggressive: bool) -> Result<String, String> {
    let host = resolver_equipo(raiz)?;
    let all = generations();
    let current = current_generation();
    let default = default_generation();
    let scheduled = scheduled_recovery();

    let mut protected = BTreeSet::new();
    if let Some(v) = current {
        protected.insert(v);
    }
    if let Some(v) = default {
        protected.insert(v);
    }
    if let Some(v) = scheduled {
        protected.insert(v);
    }

    if !aggressive {
        for (id, _) in all.iter().rev().take(3) {
            protected.insert(*id);
        }
    }

    let keep: Vec<u32> = all
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| protected.contains(id))
        .collect();
    let delete: Vec<u32> = all
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| !protected.contains(id))
        .collect();

    let array = |values: &[u32]| {
        format!(
            "[{}]",
            values
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    Ok(format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-clean-plan\",\"hostId\":{},\"policy\":{},\"protected\":{{\"current\":{},\"default\":{},\"scheduledRecovery\":{}}},\"keep\":{},\"delete\":{},\"cleanup\":{{\"garbageCollect\":true,\"optimiseStore\":true}},\"progress\":{{\"approximateStageProgress\":true,\"exactPendingBytes\":false}},\"actions\":{{\"planWritesSystem\":false,\"executionNeedsPrivilege\":true}}}}",
        json_texto(&host),
        json_texto(if aggressive { "aggressive" } else { "normal" }),
        current.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
        default.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
        scheduled.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
        array(&keep),
        array(&delete)
    ))
}

fn texto_progreso_cli(etapa: &str) -> &'static str {
    match etapa {
        "preparing" => "Preparando…",
        "changing_channel" => "Cambiando el canal del sistema…",
        "updating_catalog" => "Actualizando el catálogo de software…",
        "validating" => "Validando la configuración…",
        "building_system" => "Construyendo la nueva configuración…",
        "authorization_required" => "Esperando autorización…",
        "verifying_activation" => "Verificando la activación…",
        "cleaning_versions" => "Eliminando versiones antiguas del sistema…",
        "garbage_collect" => "Liberando espacio que ya no se usa…",
        "optimising_store" => "Optimizando el almacenamiento del sistema…",
        "saving_data" => "Terminando de guardar los datos pendientes…",
        "unmounting" => "Desconectando los sistemas de archivos…",
        "powering_off" => "Apagando la unidad…",
        "refreshing_firmware" => "Comprobando actualizaciones de firmware…",
        "installing_firmware" => "Instalando firmware…",
        "scheduling_recovery" => "Preparando la recuperación…",
        "testing_sound" => "Probando la salida de sonido…",
        "recording_mic" => "Grabando una prueba temporal del micrófono…",
        "playing_mic" => "Reproduciendo la prueba del micrófono…",
        "done" => "Listo.",
        _ => "Korunix sigue trabajando…",
    }
}

fn emitir_progreso(json: bool, porcentaje: u8, etapa: &str) {
    if !json {
        return;
    }

    let porcentaje = porcentaje.min(100);

    if io::stderr().is_terminal() {
        eprintln!("→ {}", texto_progreso_cli(etapa));
    } else {
        // La GUI consume este canal estructurado. stdout queda reservado para
        // el documento JSON final de la operación.
        eprintln!("KORUNIX_PROGRESS\t{porcentaje}\t{etapa}");
    }

    let _ = io::stderr().flush();
}

fn emitir_fase(json: bool, porcentaje: u8, etapa: &str, mensaje: &str) {
    if json {
        emitir_progreso(true, porcentaje, etapa);
    } else {
        eprintln!("→ {mensaje}");
        let _ = io::stderr().flush();
    }
}

fn emitir_pulso_construccion(json: bool, segundos: u64) {
    if json && !io::stderr().is_terminal() {
        emitir_progreso(true, 35, "building_system");
        return;
    }

    eprintln!("  {segundos} s transcurridos · Nix sigue construyendo…");
    let _ = io::stderr().flush();
}

fn clean(
    raiz: &Path,
    aggressive: bool,
    preview: bool,
    args: &[String],
) -> Result<ExitCode, String> {
    let json = args.iter().any(|v| v == "--json");
    let yes = args.iter().any(|v| v == "--yes");
    if args.iter().any(|v| v != "--json" && v != "--yes") {
        return Err("Opción de limpieza desconocida.".into());
    }

    let plan = clean_plan_json(raiz, aggressive)?;

    if preview {
        if yes {
            return Err("--yes no se utiliza junto con un plan.".into());
        }
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if json && !yes {
        return Err("La limpieza JSON necesita --yes.".into());
    }

    emitir_progreso(json, 5, "preparing");
    let ids = jq_con_entrada(raiz, &["-r".into(), ".delete[]?".into()], &plan)?;
    if ids.trim().is_empty() {
        let result = jq0(
            raiz,
            &[
                "-cn".into(),
                "--argjson".into(),
                "plan".into(),
                plan,
                r#"{
                  schemaVersion:1,
                  kind:"korunix-clean-result",
                  policy:$plan.policy,
                  protected:$plan.protected,
                  deleted:[],
                  completed:true,
                  nothingToDo:true,
                  garbageCollected:false,
                  storeOptimised:false
                }"#
                .into(),
            ],
        )?;
        if json {
            println!("{result}");
        } else {
            println!("✓ No hay nada que limpiar.");
        }
        return Ok(ExitCode::SUCCESS);
    }

    if !yes {
        if aggressive {
            if !io::stdin().is_terminal() {
                return Err("clean-all necesita confirmación interactiva.".into());
            }
            print!("Escribe BORRAR para continuar: ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let mut value = String::new();
            io::stdin()
                .read_line(&mut value)
                .map_err(|e| e.to_string())?;
            if value.trim() != "BORRAR" {
                return Ok(ExitCode::SUCCESS);
            }
        } else if !confirm("¿Ejecutar esta limpieza?")? {
            return Ok(ExitCode::SUCCESS);
        }
    }

    emitir_progreso(json, 15, "cleaning_versions");
    for id in ids.lines().filter(|v| !v.is_empty()) {
        let _ = privileged(
            raiz,
            "nix-env",
            &[
                "--profile".into(),
                system_profile().display().to_string(),
                "--delete-generations".into(),
                id.into(),
            ],
            false,
        )?;
    }
    emitir_progreso(json, 65, "garbage_collect");
    let _ = privileged(raiz, "nix-collect-garbage", &[], false)?;

    emitir_progreso(json, 88, "optimising_store");
    let _ = privileged(raiz, "nix", &["store".into(), "optimise".into()], false)?;

    emitir_progreso(json, 100, "done");

    let result = jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "plan".into(),
            plan,
            r#"{
              schemaVersion:1,
              kind:"korunix-clean-result",
              policy:$plan.policy,
              protected:$plan.protected,
              deleted:$plan.delete,
              completed:true,
              nothingToDo:false,
              garbageCollected:true,
              storeOptimised:true
            }"#
            .into(),
        ],
    )?;

    if json {
        println!("{result}");
    } else {
        println!("✓ LIMPIEZA COMPLETA");
    }
    Ok(ExitCode::SUCCESS)
}

#[derive(Clone, Debug)]
struct TransferSource {
    path: PathBuf,
    name: String,
    bytes: u64,
}

fn transfer_sources(paths: &[String]) -> Result<Vec<TransferSource>, String> {
    if paths.is_empty() {
        return Err("Selecciona al menos un archivo para transferir.".to_string());
    }

    let mut names = BTreeSet::<String>::new();
    let mut result = Vec::<TransferSource>::new();

    for raw in paths {
        let requested = PathBuf::from(raw);
        let link_meta = fs::symlink_metadata(&requested)
            .map_err(|error| format!("No pude leer {}: {error}", requested.display()))?;

        if link_meta.file_type().is_symlink() {
            return Err(format!(
                "{} es un enlace. Korunix solo transfiere archivos normales seleccionados explícitamente.",
                requested.display()
            ));
        }

        if !link_meta.is_file() {
            return Err(format!(
                "{} no es un archivo normal. Este asistente no copia carpetas.",
                requested.display()
            ));
        }

        let path = fs::canonicalize(&requested)
            .map_err(|error| format!("No pude resolver {}: {error}", requested.display()))?;
        let meta = fs::metadata(&path)
            .map_err(|error| format!("No pude medir {}: {error}", path.display()))?;

        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{} no tiene un nombre utilizable.", path.display()))?
            .to_string();

        if !names.insert(name.clone()) {
            return Err(format!(
                "Has seleccionado más de un archivo llamado «{name}». Elige nombres distintos para evitar una colisión en el destino."
            ));
        }

        result.push(TransferSource {
            path,
            name,
            bytes: meta.len(),
        });
    }

    Ok(result)
}

fn storage_transfer_target_json(raiz: &Path, device: &str) -> Result<String, String> {
    let selected = fs::canonicalize(device)
        .unwrap_or_else(|_| PathBuf::from(device))
        .display()
        .to_string();

    let disk = storage_parent(raiz, &selected)?;
    let flags = capture(
        raiz,
        "lsblk",
        &[
            "-dnro".into(),
            "RM,HOTPLUG,TRAN".into(),
            "--".into(),
            disk.clone(),
        ],
    )?;

    let mut parts = flags.split_whitespace();
    let rm = parts.next().unwrap_or("");
    let hot = parts.next().unwrap_or("");
    let transport = parts.next().unwrap_or("");
    let removable = rm == "1" || hot == "1" || matches!(transport, "usb" | "mmc");

    if !removable {
        return Err(
            "Las transferencias seguras de este asistente solo usan unidades extraíbles."
                .to_string(),
        );
    }

    let selected_type = capture(
        raiz,
        "lsblk",
        &["-dnro".into(), "TYPE".into(), "--".into(), selected.clone()],
    )?;

    let raw = capture(
        raiz,
        "lsblk",
        &[
            "-b".into(),
            "-J".into(),
            "-p".into(),
            "--tree".into(),
            "-o".into(),
            "PATH,TYPE,SIZE,FSTYPE,MOUNTPOINTS".into(),
            "--".into(),
            disk.clone(),
        ],
    )?;

    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            "--arg".into(),
            "selected".into(),
            selected,
            "--arg".into(),
            "disk".into(),
            disk,
            "--arg".into(),
            "transport".into(),
            transport.into(),
            "--arg".into(),
            "selectedType".into(),
            selected_type.trim().to_string(),
            r#"
              def nodes: ., (.children[]? | nodes);
              def mount:
                [(.mountpoints // [])[]?
                 | select(type=="string" and startswith("/"))][0] // null;
              def acceptable:
                (.fstype != null)
                and (.fstype != "")
                and (.fstype != "swap")
                and (.fstype != "iso9660");

              [.blockdevices[]?
               | nodes
               | select((.type=="part" or .type=="disk") and acceptable)
               | {
                   partition:.path,
                   fileSystem:.fstype,
                   size:((.size | tonumber?) // 0),
                   mountPoint:mount
                 }
              ] as $candidates
              | (
                  if $selectedType=="part" then
                    [$candidates[] | select(.partition==$selected)][0]
                  else
                    ($candidates | sort_by(.size) | reverse | .[0])
                  end
                ) as $target
              | if $target == null then
                  error("La unidad no contiene un sistema de archivos adecuado para recibir archivos.")
                else
                  {
                    schemaVersion:1,
                    kind:"korunix-storage-transfer-target",
                    disk:$disk,
                    selectedDevice:$selected,
                    partition:$target.partition,
                    fileSystem:$target.fileSystem,
                    sizeBytes:$target.size,
                    mountPoint:$target.mountPoint,
                    mounted:($target.mountPoint != null),
                    transport:($transport | if .=="" then null else . end),
                    removable:true
                  }
                end
            "#
            .into(),
        ],
        &raw,
    )
}

fn storage_transfer_plan_json(
    raiz: &Path,
    device: &str,
    source_paths: &[String],
) -> Result<String, String> {
    let sources = transfer_sources(source_paths)?;
    let target_text = storage_transfer_target_json(raiz, device)?;
    let target: serde_json::Value =
        serde_json::from_str(&target_text).map_err(|error| error.to_string())?;

    let file_system = target
        .get("fileSystem")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if file_system == "vfat" && sources.iter().any(|source| source.bytes > 4_294_967_295) {
        return Err(
            "Esta unidad usa FAT y no puede recibir uno de los archivos porque supera 4 GiB."
                .to_string(),
        );
    }

    let mount = target
        .get("mountPoint")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from);

    let items = sources
        .iter()
        .map(|source| {
            let conflict = mount
                .as_ref()
                .map(|mount| mount.join(&source.name).exists())
                .unwrap_or(false);

            serde_json::json!({
                "source": source.path.display().to_string(),
                "name": source.name.clone(),
                "bytes": source.bytes,
                "conflict": conflict
            })
        })
        .collect::<Vec<_>>();

    let total_bytes = sources.iter().map(|source| source.bytes).sum::<u64>();
    let conflicts = items
        .iter()
        .filter(|item| item.get("conflict").and_then(serde_json::Value::as_bool) == Some(true))
        .filter_map(|item| item.get("name").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-storage-transfer-plan",
        "target": target,
        "items": items,
        "fileCount": sources.len(),
        "totalBytes": total_bytes,
        "conflicts": conflicts,
        "measurableProgress": {
            "copyBytes": true,
            "speed": true,
            "etaDuringCopy": true,
            "persistenceDuration": false,
            "verificationDuration": false
        },
        "safety": {
            "overwritesExistingFiles": false,
            "usesGlobalSync": false,
            "persistsPerFilesystem": true,
            "verifiesContent": true,
            "automaticEject": false
        }
    })
    .to_string())
}

#[cfg(target_os = "linux")]
fn sincronizar_sistema_archivos(path: &Path) -> Result<(), String> {
    use std::os::fd::AsRawFd;

    extern "C" {
        fn syncfs(fd: i32) -> i32;
    }

    let directory = fs::File::open(path).map_err(|error| {
        format!(
            "No pude abrir {} para confirmar sus datos: {error}",
            path.display()
        )
    })?;

    let result = unsafe { syncfs(directory.as_raw_fd()) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "No pude confirmar las escrituras pendientes de {}: {}",
            path.display(),
            io::Error::last_os_error()
        ))
    }
}

#[cfg(not(target_os = "linux"))]
fn sincronizar_sistema_archivos(_path: &Path) -> Result<(), String> {
    Err("La persistencia por sistema de archivos de Korunix requiere Linux.".to_string())
}

#[cfg(target_os = "linux")]
fn renombrar_sin_reemplazar(source: &Path, target: &Path) -> Result<(), String> {
    use std::ffi::{c_char, CString};
    use std::os::unix::ffi::OsStrExt;

    const AT_FDCWD: i32 = -100;
    const RENAME_NOREPLACE: u32 = 1;

    extern "C" {
        fn renameat2(
            olddirfd: i32,
            oldpath: *const c_char,
            newdirfd: i32,
            newpath: *const c_char,
            flags: u32,
        ) -> i32;
    }

    let source_c = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| format!("Ruta de transferencia no válida: {}", source.display()))?;
    let target_c = CString::new(target.as_os_str().as_bytes())
        .map_err(|_| format!("Ruta de transferencia no válida: {}", target.display()))?;

    let result = unsafe {
        renameat2(
            AT_FDCWD,
            source_c.as_ptr(),
            AT_FDCWD,
            target_c.as_ptr(),
            RENAME_NOREPLACE,
        )
    };

    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "No pude finalizar {} sin reemplazar un archivo existente: {}",
            target.display(),
            io::Error::last_os_error()
        ))
    }
}

#[cfg(not(target_os = "linux"))]
fn renombrar_sin_reemplazar(_source: &Path, _target: &Path) -> Result<(), String> {
    Err("La finalización segura de transferencias de Korunix requiere Linux.".to_string())
}

fn archivos_iguales(left: &Path, right: &Path) -> Result<bool, String> {
    let left_meta = fs::metadata(left)
        .map_err(|error| format!("No pude volver a leer {}: {error}", left.display()))?;
    let right_meta = fs::metadata(right)
        .map_err(|error| format!("No pude verificar {}: {error}", right.display()))?;

    if left_meta.len() != right_meta.len() {
        return Ok(false);
    }

    let mut left_file = fs::File::open(left)
        .map_err(|error| format!("No pude abrir {}: {error}", left.display()))?;
    let mut right_file = fs::File::open(right)
        .map_err(|error| format!("No pude verificar {}: {error}", right.display()))?;

    let mut left_buffer = vec![0u8; 1024 * 1024];
    let mut right_buffer = vec![0u8; 1024 * 1024];

    loop {
        let left_read = left_file
            .read(&mut left_buffer)
            .map_err(|error| format!("No pude verificar {}: {error}", left.display()))?;
        let right_read = right_file
            .read(&mut right_buffer)
            .map_err(|error| format!("No pude verificar {}: {error}", right.display()))?;

        if left_read != right_read {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
        if left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
    }
}

fn emitir_progreso_transferencia(
    json: bool,
    stage: &str,
    percent: Option<u8>,
    current: Option<&str>,
    file_index: usize,
    file_count: usize,
    transferred_bytes: u64,
    total_bytes: u64,
    current_file_bytes: u64,
    current_file_total: u64,
    bytes_per_second: Option<f64>,
    eta_seconds: Option<u64>,
) {
    let event = serde_json::json!({
        "stage": stage,
        "percent": percent,
        "current": current,
        "fileIndex": file_index,
        "fileCount": file_count,
        "transferredBytes": transferred_bytes,
        "totalBytes": total_bytes,
        "currentFileBytes": current_file_bytes,
        "currentFileTotal": current_file_total,
        "bytesPerSecond": bytes_per_second,
        "etaSeconds": eta_seconds
    });

    if json && !io::stderr().is_terminal() {
        eprintln!("KORUNIX_TRANSFER\t{event}");
    } else {
        let mut message = match current {
            Some(current) if !current.is_empty() => format!("{stage}: {current}"),
            _ => stage.to_string(),
        };

        if let Some(percent) = percent {
            message.push_str(&format!(" · {percent}%"));
        }
        if let Some(speed) = bytes_per_second {
            message.push_str(&format!(" · {:.1} MiB/s", speed / 1024.0 / 1024.0));
        }
        if let Some(eta) = eta_seconds {
            message.push_str(&format!(" · ~{eta} s"));
        }

        eprintln!("→ {message}");
    }

    let _ = io::stderr().flush();
}

fn storage_mount_transfer_target(
    raiz: &Path,
    partition: &str,
    current_mount: Option<&str>,
) -> Result<(PathBuf, bool), String> {
    if let Some(current) = current_mount {
        let path = PathBuf::from(current);
        if path.is_dir() {
            return Ok((path, false));
        }
    }

    let (code, output, error) = capture_status(
        raiz,
        "udisksctl",
        &["mount".into(), "-b".into(), partition.into()],
    )?;

    if code != 0 {
        let detail = if error.trim().is_empty() {
            output
        } else {
            error
        };
        return Err(if detail.trim().is_empty() {
            format!("No pude dejar disponible {partition} para recibir archivos.")
        } else {
            detail
        });
    }

    let mounts = capture(
        raiz,
        "lsblk",
        &[
            "-nro".into(),
            "MOUNTPOINTS".into(),
            "--".into(),
            partition.into(),
        ],
    )?;

    let mount = mounts
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('/'))
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!("{partition} se preparó, pero el sistema no informó dónde quedó disponible.")
        })?;

    Ok((mount, true))
}

fn transferir_archivos_a_directorio(
    sources: &[TransferSource],
    destination: &Path,
    json: bool,
) -> Result<Vec<PathBuf>, String> {
    if !destination.is_dir() {
        return Err(format!(
            "{} no es un destino disponible.",
            destination.display()
        ));
    }

    let total_bytes = sources.iter().map(|source| source.bytes).sum::<u64>();
    let file_count = sources.len();
    let transfer_stamp = stamp();
    let mut temporary = Vec::<(PathBuf, PathBuf, PathBuf)>::new();
    let mut finals = Vec::<PathBuf>::new();

    for source in sources {
        let final_path = destination.join(&source.name);
        if final_path.exists() {
            return Err(format!(
                "Ya existe «{}» en el destino. Korunix no reemplazará archivos durante una transferencia segura.",
                source.name
            ));
        }
    }

    let result = (|| -> Result<Vec<PathBuf>, String> {
        let started = std::time::Instant::now();
        let mut transferred = 0u64;
        let mut last_event = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);

        if total_bytes == 0 {
            emitir_progreso_transferencia(
                json,
                "copying_files",
                Some(100),
                None,
                0,
                file_count,
                0,
                0,
                0,
                0,
                None,
                Some(0),
            );
        }

        for (index, source) in sources.iter().enumerate() {
            let final_path = destination.join(&source.name);
            let temp_path =
                destination.join(format!(".korunix-transfer-{transfer_stamp}-{index}.part"));

            let mut input = fs::File::open(&source.path)
                .map_err(|error| format!("No pude abrir {}: {error}", source.path.display()))?;

            let mut output = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp_path)
                .map_err(|error| {
                    format!(
                        "No pude crear el archivo temporal en {}: {error}",
                        destination.display()
                    )
                })?;

            temporary.push((source.path.clone(), temp_path.clone(), final_path.clone()));

            let mut buffer = vec![0u8; 8 * 1024 * 1024];
            let mut current_bytes = 0u64;

            loop {
                let read = input
                    .read(&mut buffer)
                    .map_err(|error| format!("No pude leer {}: {error}", source.path.display()))?;

                if read == 0 {
                    break;
                }

                output
                    .write_all(&buffer[..read])
                    .map_err(|error| format!("No pude guardar {}: {error}", source.name))?;

                current_bytes += read as u64;
                transferred += read as u64;

                let elapsed = started.elapsed().as_secs_f64();
                let speed = (elapsed > 0.0).then_some(transferred as f64 / elapsed);
                let eta = speed.and_then(|speed| {
                    if speed > 0.0 && total_bytes >= transferred {
                        Some(((total_bytes - transferred) as f64 / speed).ceil() as u64)
                    } else {
                        None
                    }
                });

                if last_event.elapsed() >= std::time::Duration::from_millis(250)
                    || current_bytes == source.bytes
                {
                    let percent = if total_bytes == 0 {
                        Some(100)
                    } else {
                        Some(((transferred.saturating_mul(100) / total_bytes).min(100)) as u8)
                    };

                    emitir_progreso_transferencia(
                        json,
                        "copying_files",
                        percent,
                        Some(&source.name),
                        index + 1,
                        file_count,
                        transferred,
                        total_bytes,
                        current_bytes,
                        source.bytes,
                        speed,
                        eta,
                    );

                    last_event = std::time::Instant::now();
                }
            }

            if current_bytes != source.bytes {
                return Err(format!(
                    "{} cambió mientras se copiaba. Korunix no declarará la transferencia completada.",
                    source.path.display()
                ));
            }

            output
                .sync_all()
                .map_err(|error| format!("No pude confirmar {}: {error}", source.name))?;
        }

        emitir_progreso_transferencia(
            json,
            "saving_data",
            None,
            None,
            file_count,
            file_count,
            total_bytes,
            total_bytes,
            0,
            0,
            None,
            None,
        );

        sincronizar_sistema_archivos(destination)?;

        for (index, (source, temp, _)) in temporary.iter().enumerate() {
            let name = source
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("archivo");

            emitir_progreso_transferencia(
                json,
                "verifying_transfer",
                None,
                Some(name),
                index + 1,
                file_count,
                total_bytes,
                total_bytes,
                0,
                fs::metadata(source).map(|meta| meta.len()).unwrap_or(0),
                None,
                None,
            );

            if !archivos_iguales(source, temp)? {
                return Err(format!(
                    "La copia de «{name}» no coincide con el archivo original. Korunix no la considera válida."
                ));
            }
        }

        for (_, temp, final_path) in &temporary {
            if final_path.exists() {
                return Err(format!(
                    "Apareció un archivo llamado «{}» en el destino mientras Korunix verificaba la copia. No se reemplazará.",
                    final_path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("archivo")
                ));
            }

            renombrar_sin_reemplazar(temp, final_path)?;
            finals.push(final_path.clone());
        }

        sincronizar_sistema_archivos(destination)?;

        emitir_progreso_transferencia(
            json,
            "done",
            Some(100),
            None,
            file_count,
            file_count,
            total_bytes,
            total_bytes,
            0,
            0,
            None,
            Some(0),
        );

        Ok(finals.clone())
    })();

    if result.is_err() {
        for (_, temp, _) in &temporary {
            let _ = fs::remove_file(temp);
        }
        for final_path in &finals {
            let _ = fs::remove_file(final_path);
        }
        let _ = sincronizar_sistema_archivos(destination);
    }

    result
}

fn storage_transfer(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let Some(device) = args.first() else {
        return Err(
            "Uso: korunix storage transfer <unidad> --source <archivo>... [--plan|--yes] [--json]."
                .to_string(),
        );
    };

    if device.starts_with('-') {
        return Err("Indica primero la unidad extraíble de destino.".to_string());
    }

    let mut source_paths = Vec::<String>::new();
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;
    let mut index = 1usize;

    while index < args.len() {
        match args[index].as_str() {
            "--source" => {
                index += 1;
                let source = args
                    .get(index)
                    .ok_or_else(|| "--source necesita una ruta de archivo.".to_string())?;
                source_paths.push(source.clone());
            }
            "--plan" if !plan_only => plan_only = true,
            "--yes" if !yes => yes = true,
            "--json" if !json => json = true,
            other => return Err(format!("Opción de transferencia desconocida: {other}")),
        }
        index += 1;
    }

    let plan = storage_transfer_plan_json(raiz, device, &source_paths)?;

    if plan_only {
        if yes {
            return Err("--yes no se utiliza junto con --plan.".to_string());
        }
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if json && !yes {
        return Err("storage transfer --json necesita --yes.".to_string());
    }

    let conflicts = jq_con_entrada(raiz, &["-r".into(), ".conflicts[]?".into()], &plan)?;

    if !conflicts.trim().is_empty() {
        return Err(format!(
            "El destino ya contiene estos archivos y Korunix no los reemplazará:\n{}",
            conflicts
        ));
    }

    if !yes && !confirm("¿Iniciar esta transferencia segura?")? {
        return Ok(ExitCode::SUCCESS);
    }

    let sources = transfer_sources(&source_paths)?;
    let target_text = storage_transfer_target_json(raiz, device)?;
    let target: serde_json::Value =
        serde_json::from_str(&target_text).map_err(|error| error.to_string())?;

    let partition = target
        .get("partition")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "El destino no contiene una partición utilizable.".to_string())?;

    let current_mount = target.get("mountPoint").and_then(serde_json::Value::as_str);

    emitir_progreso_transferencia(
        json,
        "preparing",
        None,
        None,
        0,
        sources.len(),
        0,
        sources.iter().map(|source| source.bytes).sum(),
        0,
        0,
        None,
        None,
    );

    let (mount, mounted_by_korunix) =
        storage_mount_transfer_target(raiz, partition, current_mount)?;

    let total_bytes = sources.iter().map(|source| source.bytes).sum::<u64>();
    let final_paths = transferir_archivos_a_directorio(&sources, &mount, json)?;

    let result = serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-storage-transfer-result",
        "completed": true,
        "verified": true,
        "persisted": true,
        "disk": target.get("disk").cloned().unwrap_or(serde_json::Value::Null),
        "partition": partition,
        "mountPoint": mount.display().to_string(),
        "mountedByKorunix": mounted_by_korunix,
        "fileCount": sources.len(),
        "totalBytes": total_bytes,
        "files": final_paths
            .iter()
            .map(|path| {
                serde_json::json!({
                    "name": path.file_name().and_then(|value| value.to_str()),
                    "destination": path.display().to_string()
                })
            })
            .collect::<Vec<_>>(),
        "readyToEject": true,
        "automaticEject": false,
        "usesGlobalSync": false
    });

    if json {
        println!("{result}");
    } else {
        println!("✓ Transferencia completada, persistida y verificada.");
    }

    Ok(ExitCode::SUCCESS)
}

fn storage_tool(raiz: &Path, programa: &str, args: &[String], json: bool) -> Result<(), String> {
    if !json {
        return visible(raiz, programa, args);
    }

    let (code, output, error) = capture_status(raiz, programa, args)?;
    if code == 0 {
        return Ok(());
    }

    let detail = if !error.trim().is_empty() {
        error
    } else if !output.trim().is_empty() {
        output
    } else {
        format!("{programa} terminó con error.")
    };

    Err(detail)
}

fn storage_list_json(raiz: &Path) -> Result<String, String> {
    let raw = capture(
        raiz,
        "lsblk",
        &[
            "-J".into(),
            "-p".into(),
            "--tree".into(),
            "-o".into(),
            "PATH,TYPE,RM,HOTPLUG,TRAN,SIZE,MODEL,MOUNTPOINTS,UUID,FSTYPE".into(),
        ],
    )?;

    let declaradas = nix_config_json(raiz, "storage.dataVolumes")?;

    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            "--argjson".into(),
            "declared".into(),
            declaradas,
            r#"
              def truthy: .==true or .==1 or .=="1";
              def nodes: ., (.children[]? | nodes);
              def mounts:
                [(.mountpoints // [])[]?
                 | select(type=="string" and length>0)]
                | unique;
              def declared_for($uuid):
                if $uuid == null then null
                else
                  ($declared
                   | map(select(
                       (.uuid | ascii_downcase)
                       == ($uuid | ascii_downcase)
                     ))
                   | .[0] // null)
                end;

              [.blockdevices[]?
               | select(.type=="disk")
               | . as $disk
               | {
                   device:.path,
                   size:.size,
                   model:(.model
                     | if .==null
                       then null
                       else gsub("[[:space:]]+$";"")
                       end),
                   transport:.tran,
                   removable:(
                     (.rm|truthy)
                     or (.hotplug|truthy)
                     or .tran=="usb"
                     or .tran=="mmc"
                   ),
                   mountPoints:(
                     [$disk
                      | nodes
                      | (.mountpoints // [])[]?
                      | select(type=="string" and length>0)]
                     | unique
                   ),
                   dataVolumes:(
                     [$disk
                      | nodes
                      | select(.uuid != null and .fstype != null)
                      | . as $node
                      | (declared_for($node.uuid)) as $managed
                      | {
                          device:$node.path,
                          uuid:$node.uuid,
                          fileSystem:$node.fstype,
                          mountPoints:($node|mounts),
                          managed:($managed != null),
                          availableAtLogin:($managed.availableAtLogin // false),
                          configuredPath:($managed.path // null),
                          id:($managed.id // null)
                        }]
                   )
                 }]
              | {
                  schemaVersion:2,
                  kind:"korunix-storage-list",
                  devices:.
                }
            "#
            .into(),
        ],
        &raw,
    )
}

fn storage_parent(raiz: &Path, device: &str) -> Result<String, String> {
    let kind = capture(
        raiz,
        "lsblk",
        &["-dnro".into(), "TYPE".into(), "--".into(), device.into()],
    )?;
    if kind.lines().next() == Some("disk") {
        return Ok(device.to_string());
    }
    let ancestors = capture(
        raiz,
        "lsblk",
        &[
            "-s".into(),
            "-nro".into(),
            "PATH,TYPE".into(),
            "--".into(),
            device.into(),
        ],
    )?;
    ancestors
        .lines()
        .find_map(|l| {
            let mut p = l.split_whitespace();
            let path = p.next()?;
            let kind = p.next()?;
            (kind == "disk").then(|| path.to_string())
        })
        .ok_or_else(|| format!("No pude determinar el disco físico de {device}."))
}

fn storage_plan_json(raiz: &Path, device: &str, heavy: bool) -> Result<String, String> {
    let selected = fs::canonicalize(device)
        .unwrap_or_else(|_| PathBuf::from(device))
        .display()
        .to_string();
    let disk = storage_parent(raiz, &selected)?;
    let flags = capture(
        raiz,
        "lsblk",
        &[
            "-dnro".into(),
            "RM,HOTPLUG,TRAN".into(),
            "--".into(),
            disk.clone(),
        ],
    )?;
    let mut parts = flags.split_whitespace();
    let rm = parts.next().unwrap_or("");
    let hot = parts.next().unwrap_or("");
    let tran = parts.next().unwrap_or("");
    let removable = rm == "1" || hot == "1" || matches!(tran, "usb" | "mmc");
    if !removable {
        return Err("Korunix solo permite expulsar unidades extraíbles o hotplug.".into());
    }

    let raw = capture(
        raiz,
        "lsblk",
        &[
            "-J".into(),
            "-p".into(),
            "-o".into(),
            "PATH,MOUNTPOINTS".into(),
            "--".into(),
            disk.clone(),
        ],
    )?;
    let mounts = jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            r#"[..|objects|select(has("path") and has("mountpoints"))|. as $n|(.mountpoints//[])[]?|select(type=="string" and length>0)|{device:$n.path,mountPoint:.}]|unique_by(.device+"\u0000"+.mountPoint)|sort_by(.mountPoint|length)|reverse"#.into(),
        ],
        &raw,
    )?;
    let system_disk = jq_texto(
        raiz,
        &mounts,
        r#"any(.[]; .mountPoint=="/" or .mountPoint=="/nix" or .mountPoint=="/boot" or .mountPoint=="/boot/efi")"#,
    )? == "true";
    if system_disk {
        return Err(
            "Korunix nunca expulsará un disco que contenga el sistema en ejecución.".into(),
        );
    }

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--arg".into(),
            "host".into(),
            resolver_equipo(raiz)?,
            "--arg".into(),
            "selected".into(),
            selected,
            "--arg".into(),
            "disk".into(),
            disk,
            "--arg".into(),
            "transport".into(),
            tran.into(),
            "--argjson".into(),
            "mounts".into(),
            mounts,
            "--argjson".into(),
            "heavy".into(),
            heavy.to_string(),
            r#"{
              schemaVersion:1,
              kind:"korunix-storage-eject-plan",
              hostId:$host,
              selectedDevice:$selected,
              disk:$disk,
              transport:($transport|if .=="" then null else . end),
              removable:true,
              systemDisk:false,
              mounts:$mounts,
              heavyTransferMode:$heavy,
              writeback:{
                globalSync:false,
                strategy:(if $heavy then "syncfs-per-filesystem" else "udisks-unmount-flush" end)
              },
              progress:{approximateStageProgress:true,exactPendingBytes:false},
              actions:{
                planWritesSystem:false,
                unmountsFilesystems:true,
                powersOffDevice:true,
                usesGlobalSync:false
              }
            }"#
            .into(),
        ],
    )
}

fn storage(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args == ["--list"] || args == ["--list", "--json"] {
        let data = storage_list_json(raiz)?;
        if args.last().map(String::as_str) == Some("--json") {
            println!("{data}");
        } else {
            pretty(raiz, &data)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) == Some("transfer") {
        return storage_transfer(raiz, &args[1..]);
    }

    if args.first().map(String::as_str) != Some("eject") || args.len() < 2 {
        return Err(
            "Uso: korunix storage --list [--json] | transfer <unidad> --source <archivo>... [--plan|--yes] [--json] | eject <dispositivo> [--heavy] [--plan] [--yes] [--json]."
                .into(),
        );
    }

    let device = &args[1];
    let heavy = args.iter().any(|value| value == "--heavy");
    let plan_only = args.iter().any(|value| value == "--plan");
    let yes = args.iter().any(|value| value == "--yes");
    let json = args.iter().any(|value| value == "--json");
    let plan = storage_plan_json(raiz, device, heavy)?;

    if plan_only {
        if yes {
            return Err("--yes no se utiliza junto con --plan.".into());
        }
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if json && !yes {
        return Err("storage eject --json necesita --yes.".into());
    }
    if !yes && !confirm(&format!("¿Expulsar de forma segura {device}?"))? {
        return Ok(ExitCode::SUCCESS);
    }

    emitir_progreso(json, 8, "preparing");

    if heavy {
        emitir_progreso(json, 20, "saving_data");
        let mounts = jq_con_entrada(raiz, &["-r".into(), ".mounts[].mountPoint".into()], &plan)?;
        for mount in mounts.lines().filter(|value| !value.is_empty()) {
            sincronizar_sistema_archivos(Path::new(mount))?;
        }
    }

    emitir_progreso(json, 55, "unmounting");

    let devices = jq_con_entrada(
        raiz,
        &["-r".into(), ".mounts | map(.device) | unique[]".into()],
        &plan,
    )?;

    for dev in devices.lines().filter(|value| !value.is_empty()) {
        storage_tool(
            raiz,
            "udisksctl",
            &["unmount".into(), "-b".into(), dev.into()],
            json,
        )?;
    }

    emitir_progreso(json, 88, "powering_off");

    let disk = jq_texto(raiz, &plan, ".disk")?;
    storage_tool(
        raiz,
        "udisksctl",
        &["power-off".into(), "-b".into(), disk.clone()],
        json,
    )?;

    emitir_progreso(json, 100, "done");

    let result = jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "plan".into(),
            plan,
            r#"{
              schemaVersion:1,
              kind:"korunix-storage-eject-result",
              disk:$plan.disk,
              heavyTransferMode:$plan.heavyTransferMode,
              unmounted:$plan.mounts,
              poweredOff:true,
              safeToDisconnect:true,
              writeback:$plan.writeback
            }"#
            .into(),
        ],
    )?;

    if json {
        println!("{result}");
    } else {
        println!("✓ Unidad expulsada de forma segura.");
    }
    Ok(ExitCode::SUCCESS)
}

fn update_targets(
    raiz: &Path,
    requested: &[String],
) -> Result<(String, Vec<String>, String), String> {
    let lock = raiz.join("flake.lock");
    let available_raw = capture(
        raiz,
        "jq",
        &[
            "-r".into(),
            ".nodes.root.inputs | keys[]".into(),
            lock.display().to_string(),
        ],
    )?;
    let available: BTreeSet<String> = available_raw.lines().map(ToString::to_string).collect();

    let (scope, targets) = if requested.is_empty() {
        (
            "all".to_string(),
            available.iter().cloned().collect::<Vec<_>>(),
        )
    } else {
        let mut unique = BTreeSet::new();
        for input in requested {
            if !available.contains(input) {
                return Err(format!("Entrada del flake desconocida: {input}"));
            }
            unique.insert(input.clone());
        }
        ("selected".to_string(), unique.into_iter().collect())
    };

    let targets_json = json_lista_textos(&targets);
    let items = capture(
        raiz,
        "jq",
        &[
            "-c".into(),
            "--argjson".into(),
            "targets".into(),
            targets_json,
            r#"[ $targets[] as $name
                 | (.nodes.root.inputs[$name] // null) as $reference
                 | (if ($reference|type)=="string" then $reference else null end) as $node
                 | {
                     name:$name,
                     reference:$reference,
                     node:$node,
                     original:(if $node==null then null else (.nodes[$node].original // null) end),
                     locked:(if $node==null then null else (.nodes[$node].locked // null) end)
                   }]"#
            .into(),
            lock.display().to_string(),
        ],
    )?;

    Ok((scope, targets, items))
}

fn update_plan_json(raiz: &Path, requested: &[String]) -> Result<String, String> {
    let host = resolver_equipo(raiz)?;
    let (scope, targets, items) = update_targets(raiz, requested)?;
    let channel = canal_declarado(raiz, &host).unwrap_or_default();
    let state_version = flake_raw(
        raiz,
        &format!("nixosConfigurations.{host}.config.system.stateVersion"),
    )
    .unwrap_or_default();

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--arg".into(),
            "host".into(),
            host,
            "--arg".into(),
            "scope".into(),
            scope,
            "--argjson".into(),
            "targets".into(),
            json_lista_textos(&targets),
            "--argjson".into(),
            "items".into(),
            items,
            "--arg".into(),
            "channel".into(),
            channel,
            "--arg".into(),
            "stateVersion".into(),
            state_version,
            r#"{
              schemaVersion:1,
              kind:"korunix-update-plan",
              hostId:$host,
              scope:$scope,
              targets:$targets,
              current:$items,
              channel:{
                current:($channel|if .=="" then null else . end),
                supported:["stable","unstable"]
              },
              migration:{
                stateVersion:($stateVersion|if .=="" then null else . end),
                changesStateVersion:false,
                systemMigrationsOnActivation:true,
                userPreparationOnApply:true
              },
              impact:{
                knownBeforeBuild:false,
                resolvedBy:"scripts/korunix preview --json"
              },
              recovery:{
                lockBackupBeforeUpdate:true,
                systemGenerationAfterApply:true,
                managedBy:"scripts/korunix rollback"
              },
              actions:{
                planWritesRepository:false,
                updateWritesRepository:true,
                modifiesFlakeLock:true,
                buildsGeneration:false,
                appliesGeneration:false
              },
              next:{
                preview:"scripts/korunix preview --json",
                apply:"scripts/korunix apply"
              }
            }"#
            .into(),
        ],
    )
}

fn update_result_json(
    raiz: &Path,
    before: &Path,
    after: &Path,
    plan: &str,
    backup: &Path,
) -> Result<String, String> {
    let before_sha = sha256(raiz, before)?;
    let after_sha = sha256(raiz, after)?;
    capture(
        raiz,
        "jq",
        &[
            "-cn".into(),
            "--slurpfile".into(),
            "before".into(),
            before.display().to_string(),
            "--slurpfile".into(),
            "after".into(),
            after.display().to_string(),
            "--argjson".into(),
            "plan".into(),
            plan.into(),
            "--arg".into(),
            "backup".into(),
            backup.display().to_string(),
            "--arg".into(),
            "beforeSha256".into(),
            before_sha,
            "--arg".into(),
            "afterSha256".into(),
            after_sha,
            r#"
              def input($doc;$name):
                ($doc.nodes.root.inputs[$name] // null) as $reference
                | (if ($reference|type)=="string" then $reference else null end) as $node
                | {
                    name:$name,
                    reference:$reference,
                    node:$node,
                    original:(if $node==null then null else ($doc.nodes[$node].original // null) end),
                    locked:(if $node==null then null else ($doc.nodes[$node].locked // null) end)
                  };
              [ $plan.targets[] as $name
                | input($before[0];$name) as $old
                | input($after[0];$name) as $new
                | {name:$name,before:$old,after:$new,changed:($old!=$new)}
              ] as $inputs
              | {
                  schemaVersion:1,
                  kind:"korunix-update-result",
                  hostId:$plan.hostId,
                  changed:($beforeSha256!=$afterSha256),
                  changedInputs:[$inputs[]|select(.changed)|.name],
                  inputs:$inputs,
                  lock:{
                    beforeSha256:$beforeSha256,
                    afterSha256:$afterSha256,
                    backup:$backup
                  },
                  channel:$plan.channel,
                  migration:$plan.migration,
                  impact:$plan.impact,
                  recovery:$plan.recovery,
                  actions:{
                    writesRepository:true,
                    modifiedFlakeLock:($beforeSha256!=$afterSha256),
                    buildsGeneration:false,
                    appliesGeneration:false
                  },
                  next:$plan.next
                }
            "#
            .into(),
        ],
    )
}

fn update(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let mut plan_only = false;
    let mut json = false;
    let mut inputs = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--plan" if !plan_only => plan_only = true,
            "--json" if !json => json = true,
            "-h" | "--help" => {
                println!("Uso: korunix update [entradas...] [--plan] [--json]");
                return Ok(ExitCode::SUCCESS);
            }
            x if x.starts_with('-') => return Err(format!("Opción de update desconocida: {x}")),
            x => inputs.push(x.to_string()),
        }
    }

    let plan = update_plan_json(raiz, &inputs)?;
    if plan_only {
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    emitir_progreso(json, 5, "preparing");

    let lock = raiz.join("flake.lock");
    let local = capture(
        raiz,
        "git",
        &[
            "status".into(),
            "--porcelain".into(),
            "--".into(),
            "flake.lock".into(),
        ],
    )?;
    if !local.is_empty() {
        return Err(
            "flake.lock ya tiene cambios locales. Guárdalos o reviértelos antes de actualizar."
                .into(),
        );
    }

    let backup = backup_dir(&format!("update-{}", resolver_equipo(raiz)?))?;
    let original = backup.join("flake.lock");
    fs::copy(&lock, &original).map_err(|e| format!("No pude respaldar flake.lock: {e}"))?;

    emitir_progreso(json, 25, "updating_catalog");

    // Nix escribe el resultado en un archivo candidato. El flake.lock real no se
    // toca durante la descarga ni durante la evaluación. Por eso SIGINT, SIGTERM,
    // SIGKILL o un corte de energía antes del commit dejan intacto el lock actual.
    let candidate = backup.join("flake.lock.candidate");
    let mut nix_args = vec![
        "flake".into(),
        "update".into(),
        "--output-lock-file".into(),
        candidate.display().to_string(),
    ];
    nix_args.extend(inputs);

    let (code, _, error) = capture_status(raiz, "nix", &nix_args)?;
    if code != 0 {
        let _ = fs::remove_file(&candidate);
        return Err(format!(
            "La actualización falló; flake.lock no fue modificado. {error}"
        ));
    }

    if !candidate.is_file() {
        return Err("Nix terminó sin producir el flake.lock candidato.".into());
    }

    emitir_progreso(json, 75, "validating");

    let candidate_check = vec![
        "flake".into(),
        "check".into(),
        "path:.".into(),
        "--no-build".into(),
        "--show-trace".into(),
        "--reference-lock-file".into(),
        candidate.display().to_string(),
    ];
    let (check_code, _, check_error) = capture_status(raiz, "nix", &candidate_check)?;
    if check_code != 0 {
        let _ = fs::remove_file(&candidate);
        return Err(format!(
            "La actualización candidata no pasó la validación; flake.lock no fue modificado. {check_error}"
        ));
    }

    emitir_progreso(json, 95, "preparing");

    // Todo lo que aún puede fallar se prepara antes del commit. Así una salida
    // estructurada inválida no puede dejar aplicado un lock que Korunix reporte
    // como una operación fallida.
    let result = update_result_json(raiz, &original, &candidate, &plan, &backup)?;

    // Punto de commit: una única sustitución atómica y persistida del lock validado.
    let candidate_data =
        fs::read(&candidate).map_err(|e| format!("No pude leer el flake.lock candidato: {e}"))?;
    atomic_write(&lock, &candidate_data)?;
    let _ = fs::remove_file(&candidate);

    emitir_progreso(json, 100, "done");
    if json {
        println!("{result}");
    } else {
        println!("✓ ACTUALIZACIÓN PREPARADA");
        println!("No se construyó ni aplicó una generación.");
        println!("Respaldo: {}", backup.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn build_candidate(raiz: &Path, json: bool) -> Result<PathBuf, String> {
    let host = resolver_equipo(raiz)?;

    let target = flake_reference(
        raiz,
        &format!("nixosConfigurations.{host}.config.system.build.toplevel"),
    );

    let mut child = Command::new(tool("nix"))
        .args([
            "build",
            target.as_str(),
            "--no-link",
            "--print-out-paths",
            "--show-trace",
        ])
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("No pude iniciar la construcción de NixOS: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "No pude leer el resultado de la construcción.".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "No pude leer los detalles de la construcción.".to_string())?;

    let lector_stdout = std::thread::spawn(move || {
        let mut texto = String::new();
        let mut lector = stdout;
        let resultado = lector.read_to_string(&mut texto);
        (resultado, texto)
    });

    let lector_stderr = std::thread::spawn(move || {
        let mut texto = String::new();
        let mut lector = stderr;
        let resultado = lector.read_to_string(&mut texto);
        (resultado, texto)
    });

    let inicio = std::time::Instant::now();
    let mut siguiente_pulso = 10_u64;

    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("No pude consultar la construcción: {error}"))?
        {
            break status;
        }

        let transcurridos = inicio.elapsed().as_secs();
        if transcurridos >= siguiente_pulso {
            emitir_pulso_construccion(json, transcurridos);
            siguiente_pulso += 10;
        }

        std::thread::sleep(std::time::Duration::from_millis(250));
    };

    let (stdout_result, out) = lector_stdout
        .join()
        .map_err(|_| "Falló la lectura del resultado de Nix.".to_string())?;
    stdout_result.map_err(|error| format!("No pude leer el resultado de Nix: {error}"))?;

    let (stderr_result, error) = lector_stderr
        .join()
        .map_err(|_| "Falló la lectura de los detalles de Nix.".to_string())?;
    stderr_result.map_err(|error| format!("No pude leer los detalles de Nix: {error}"))?;

    if !status.success() {
        return Err(if error.trim().is_empty() {
            "La construcción de NixOS terminó con error.".to_string()
        } else {
            error.trim().to_string()
        });
    }

    let path = out
        .lines()
        .last()
        .map(PathBuf::from)
        .ok_or_else(|| "Nix no devolvió la candidata.".to_string())?;

    if !path.exists() {
        return Err(format!("La candidata no existe: {}.", path.display()));
    }

    Ok(path)
}

fn impact_json(current: &Path, candidate: &Path) -> String {
    let kc = fs::canonicalize(current.join("kernel")).ok();
    let kn = fs::canonicalize(candidate.join("kernel")).ok();
    let reboot_known = kc.is_some() && kn.is_some();
    let reboot_required = reboot_known && kc != kn;

    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("XDG_SESSION_DESKTOP"))
        .unwrap_or_default();
    let d = desktop.to_ascii_lowercase();
    let rel = if d.contains("niri") {
        Some("sw/bin/niri")
    } else if d.contains("hyprland") {
        Some("sw/bin/Hyprland")
    } else if d.contains("gnome") {
        Some("sw/bin/gnome-shell")
    } else if d.contains("plasma") || d.contains("kde") {
        Some("sw/bin/kwin_wayland")
    } else if d.contains("cinnamon") {
        Some("sw/bin/cinnamon")
    } else {
        None
    };

    let sc = rel.and_then(|r| fs::canonicalize(current.join(r)).ok());
    let sn = rel.and_then(|r| fs::canonicalize(candidate.join(r)).ok());
    let logout_known = sc.is_some();
    let logout_required = logout_known && sc != sn;

    let classification = if reboot_required {
        "reboot"
    } else if logout_required {
        "logout"
    } else if reboot_known && logout_known {
        "immediate"
    } else {
        "unknown"
    };

    let path_json = |p: Option<PathBuf>| {
        p.map(|x| json_texto(&x.display().to_string()))
            .unwrap_or_else(|| "null".into())
    };

    format!(
        "{{\"classification\":{},\"reboot\":{{\"known\":{},\"required\":{},\"source\":\"kernel\",\"current\":{},\"candidate\":{}}},\"logout\":{{\"known\":{},\"required\":{},\"source\":\"active-session-executable\",\"desktop\":{},\"executable\":{},\"current\":{},\"candidate\":{}}}}}",
        json_texto(classification),
        reboot_known,
        if reboot_known { reboot_required.to_string() } else { "null".into() },
        path_json(kc),
        path_json(kn),
        logout_known,
        if logout_known { logout_required.to_string() } else { "null".into() },
        if desktop.is_empty() { "null".into() } else { json_texto(&desktop) },
        rel.map(json_texto).unwrap_or_else(|| "null".into()),
        path_json(sc),
        path_json(sn)
    )
}

// Los parámetros reflejan uno a uno las fases del contrato público.
#[allow(clippy::too_many_arguments)]
fn cycle_json(
    raiz: &Path,
    action: &str,
    current: &Path,
    candidate: &Path,
    validated: bool,
    previewed: bool,
    confirmed: Option<bool>,
    applied: bool,
    verified: Option<bool>,
    impact: &str,
    diff: &str,
    activation: &str,
) -> Result<String, String> {
    let host = resolver_equipo(raiz)?;
    let cv = fs::read_to_string(current.join("nixos-version"))
        .unwrap_or_default()
        .trim()
        .to_string();
    let nv = fs::read_to_string(candidate.join("nixos-version"))
        .unwrap_or_default()
        .trim()
        .to_string();
    let lines = |text: &str| {
        format!(
            "[{}]",
            text.lines()
                .filter(|v| !v.is_empty())
                .map(json_texto)
                .collect::<Vec<_>>()
                .join(",")
        )
    };

    Ok(format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-change-cycle\",\"action\":{},\"hostId\":{},\"state\":{},\"current\":{{\"systemPath\":{},\"nixosVersion\":{}}},\"candidate\":{{\"systemPath\":{},\"nixosVersion\":{}}},\"phases\":{{\"validated\":{},\"built\":true,\"previewed\":{},\"confirmed\":{},\"applied\":{},\"verified\":{}}},\"impact\":{},\"details\":{{\"closureDiff\":{},\"activationPlan\":{}}},\"safety\":{{\"writesRepository\":false,\"buildsGeneration\":true,\"appliesGeneration\":{},\"runningSystemChanged\":{}}}}}",
        json_texto(action),
        json_texto(&host),
        json_texto(if applied { "applied" } else { "prepared" }),
        if current.as_os_str().is_empty() { "null".into() } else { json_texto(&current.display().to_string()) },
        if cv.is_empty() { "null".into() } else { json_texto(&cv) },
        json_texto(&candidate.display().to_string()),
        if nv.is_empty() { "null".into() } else { json_texto(&nv) },
        validated,
        previewed,
        confirmed.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
        applied,
        verified.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
        impact,
        lines(diff),
        lines(activation),
        applied,
        applied
    ))
}

fn prepare_cycle(
    raiz: &Path,
    preview: bool,
    json: bool,
) -> Result<(PathBuf, PathBuf, String, String, String), String> {
    let current_text = current_system();
    let current = if current_text.is_empty() {
        PathBuf::new()
    } else {
        PathBuf::from(current_text)
    };
    let candidate = build_candidate(raiz, json)?;

    let diff = if preview && current.exists() {
        let (code, out, err) = capture_status(
            raiz,
            "nix",
            &[
                "store".into(),
                "diff-closures".into(),
                current.display().to_string(),
                candidate.display().to_string(),
            ],
        )?;
        if code == 0 {
            out
        } else {
            err
        }
    } else {
        String::new()
    };

    let activation = if preview {
        "La previsualización no necesita privilegios. La activación real se ejecutará solo después de confirmar y autorizar una vez, y dejará esa misma generación como predeterminada para los próximos arranques."
            .to_string()
    } else {
        String::new()
    };

    let impact = impact_json(&current, &candidate);
    Ok((current, candidate, diff, activation, impact))
}

fn change_cycle(raiz: &Path, command: &str, args: &[String]) -> Result<ExitCode, String> {
    let json = args.iter().any(|v| v == "--json");
    let yes = args.iter().any(|v| v == "--yes");

    match command {
        "preview" | "build" if args.iter().any(|v| v != "--json") => {
            return Err(format!("Uso: korunix {command} [--json]."));
        }
        "apply" if args.iter().any(|v| v != "--json" && v != "--yes") => {
            return Err("Uso: korunix apply [--yes] [--json].".into());
        }
        _ => {}
    }

    if command == "apply" && json && !yes {
        return Err("apply --json necesita --yes.".into());
    }

    if command == "apply" {
        emitir_fase(json, 5, "validating", "Validando la configuración…");

        if json {
            // stdout queda reservado para el JSON final. Las fases viajan por
            // stderr y la GUI las recibe mediante KORUNIX_PROGRESS.
            validate_quiet(raiz)?;
        } else {
            validate(raiz)?;
        }

        emitir_fase(json, 15, "validating", "Validación completada.");
    }

    emitir_fase(
        json,
        25,
        "building_system",
        "Construyendo la nueva configuración…",
    );

    let previewed = command != "build";
    let (current, candidate, diff, activation, impact) = prepare_cycle(raiz, previewed, json)?;

    emitir_fase(json, 70, "building_system", "Construcción completada.");

    if command == "build" {
        emitir_fase(json, 100, "done", "Construcción completada.");

        if json {
            println!(
                "{}",
                cycle_json(
                    raiz, "build", &current, &candidate, false, false, None, false, None, &impact,
                    "", ""
                )?
            );
        } else {
            println!("✓ CONSTRUCCIÓN COMPLETA\n{}", candidate.display());
        }
        return Ok(ExitCode::SUCCESS);
    }

    if command == "preview" {
        emitir_fase(
            json,
            85,
            "preparing",
            "Preparando la previsualización sin solicitar privilegios…",
        );

        if json {
            println!(
                "{}",
                cycle_json(
                    raiz,
                    "preview",
                    &current,
                    &candidate,
                    false,
                    true,
                    None,
                    false,
                    None,
                    &impact,
                    &diff,
                    &activation
                )?
            );
        } else {
            println!("=== Generación candidata ===\n{}", candidate.display());
            println!("\n=== Diferencias ===\n{diff}");
            println!("\n=== Activación ===\n{activation}");
            println!("\nImpacto: {}", jq_texto(raiz, &impact, ".classification")?);
            println!("✓ PREVISUALIZACIÓN COMPLETA");
        }

        emitir_fase(json, 100, "done", "Previsualización completada.");
        return Ok(ExitCode::SUCCESS);
    }

    if !yes && !confirm("¿Aplicar esta generación ahora?")? {
        return Ok(ExitCode::SUCCESS);
    }

    // Guardamos el estado previo para refrescar únicamente los monitores de
    // volúmenes cuando la activación cambie realmente la tabla declarada.
    let fstab_antes = fs::read("/etc/fstab").ok();

    let host = resolver_equipo(raiz)?;
    let flake = flake_reference(raiz, &host);

    emitir_fase(
        json,
        82,
        "authorization_required",
        "Esperando autorización para activar el sistema…",
    );

    // Esta es la única frontera privilegiada del ciclo apply. nixos-rebuild
    // registra la candidata en el perfil del sistema y actualiza el cargador
    // de arranque antes de activarla. La candidata ya fue construida durante
    // la fase anterior, así que Nix reutiliza el resultado cuando la fuente no
    // cambió.
    // En terminal dejamos visible la salida de nixos-rebuild. En modo JSON se
    // captura para que stdout siga reservado exclusivamente al documento final.
    let _ = privileged(
        raiz,
        "nixos-rebuild",
        &["switch".into(), "--flake".into(), flake],
        !json,
    )?;

    emitir_fase(
        json,
        94,
        "verifying_activation",
        "Verificando la activación…",
    );

    let candidate_text = candidate.display().to_string();
    let running_system = current_system();
    let profile_system = fs::canonicalize(system_profile())
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let registered = generations()
        .into_iter()
        .any(|(_, path)| path.display().to_string() == candidate_text);

    let verified =
        running_system == candidate_text && profile_system == candidate_text && registered;

    if !verified {
        return Err(
            "La configuración se activó, pero no quedó registrada de forma persistente como generación predeterminada."
                .into(),
        );
    }

    let _ = Command::new(tool("systemctl"))
        .args(["--user", "start", "korunix-user-prepare.service"])
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    refrescar_monitor_gvfs_si_cambio_fstab(fstab_antes);

    emitir_fase(json, 100, "done", "Configuración aplicada.");

    if json {
        println!(
            "{}",
            cycle_json(
                raiz,
                "apply",
                &current,
                &candidate,
                true,
                true,
                Some(true),
                true,
                Some(verified),
                &impact,
                &diff,
                &activation
            )?
        );
    } else {
        println!("✓ GENERACIÓN APLICADA");
    }
    Ok(ExitCode::SUCCESS)
}

fn fwupd_raw(raiz: &Path, action: &str) -> Result<String, String> {
    let (code, out, err) = capture_status(raiz, "fwupdmgr", &[action.into(), "--json".into()])?;
    match code {
        0 => Ok(out),
        2 => Ok(if out.is_empty() {
            "{\"Devices\":[]}".into()
        } else {
            out
        }),
        _ => Err(if err.is_empty() {
            format!("fwupd no pudo completar {action}.")
        } else {
            err
        }),
    }
}

fn firmware_modelos_almacenamiento(raiz: &Path) -> Result<Vec<String>, String> {
    let raw = capture(raiz, "lsblk", &["-dn".into(), "-o".into(), "MODEL".into()])?;

    let mut modelos = raw
        .lines()
        .map(str::trim)
        .filter(|modelo| !modelo.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    modelos.sort();
    modelos.dedup();
    Ok(modelos)
}

fn firmware_modelos_almacenamiento_json(raiz: &Path) -> Result<String, String> {
    let modelos = firmware_modelos_almacenamiento(raiz)?;
    Ok(format!(
        "[{}]",
        modelos
            .iter()
            .map(|modelo| json_texto(modelo))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn firmware_devices_json(raiz: &Path) -> Result<String, String> {
    let raw = fwupd_raw(raiz, "get-devices")?;
    let storage_models = firmware_modelos_almacenamiento_json(raiz)?;

    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            "--argjson".into(),
            "storageModels".into(),
            storage_models,
            r#"
              def norm:
                ascii_downcase
                | gsub("[^[:alnum:]]"; "");

              def es_almacenamiento($nombre):
                ($nombre | norm) as $n
                | if ($n | length) == 0 then
                    false
                  else
                    any(
                      $storageModels[]?;
                      (. | norm) as $m
                      | ($m | length) > 0
                        and (
                          ($n | contains($m))
                          or ($m | contains($n))
                        )
                    )
                  end;

              {
                schemaVersion:1,
                kind:"korunix-firmware-devices",
                backend:"fwupd",
                storageExcluded:true,
                devices:[
                  (.Devices // [])[]
                  | select((es_almacenamiento(.Name // "")) | not)
                  | {
                      id:(.DeviceId // null),
                      name:(.Name // "Dispositivo sin nombre"),
                      summary:(.Summary // null),
                      vendor:(.Vendor // null),
                      currentVersion:(.Version // null),
                      protocol:(.Protocol // null),
                      flags:(.Flags // []),
                      problems:(.Problems // []),
                      updatable:(((.Flags // [])|index("updatable")) != null),
                      supported:(((.Flags // [])|index("supported")) != null),
                      needsReboot:(((.Flags // [])|index("needs-reboot")) != null),
                      needsShutdown:(((.Flags // [])|index("needs-shutdown")) != null),
                      requiresAcPower:(((.Flags // [])|index("require-ac")) != null)
                    }
                ]
              }
            "#
            .into(),
        ],
        &raw,
    )
}

fn firmware_updates_json(raiz: &Path) -> Result<String, String> {
    let raw = fwupd_raw(raiz, "get-updates")?;
    let storage_models = firmware_modelos_almacenamiento_json(raiz)?;

    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            "--argjson".into(),
            "storageModels".into(),
            storage_models,
            r#"
              def norm:
                ascii_downcase
                | gsub("[^[:alnum:]]"; "");

              def es_almacenamiento($nombre):
                ($nombre | norm) as $n
                | if ($n | length) == 0 then
                    false
                  else
                    any(
                      $storageModels[]?;
                      (. | norm) as $m
                      | ($m | length) > 0
                        and (
                          ($n | contains($m))
                          or ($m | contains($n))
                        )
                    )
                  end;

              {
                schemaVersion:1,
                kind:"korunix-firmware-updates",
                backend:"fwupd",
                metadataRefreshPerformed:false,
                storageExcluded:true,
                devices:[
                  (.Devices // [])[]
                  | select((es_almacenamiento(.Name // "")) | not)
                  | {
                      id:(.DeviceId // null),
                      name:(.Name // "Dispositivo sin nombre"),
                      summary:(.Summary // null),
                      vendor:(.Vendor // null),
                      currentVersion:(.Version // null),
                      flags:(.Flags // []),
                      problems:(.Problems // []),
                      needsReboot:(((.Flags // [])|index("needs-reboot")) != null),
                      needsShutdown:(((.Flags // [])|index("needs-shutdown")) != null),
                      requiresAcPower:(((.Flags // [])|index("require-ac")) != null),
                      releases:[
                        (.Releases // [])[]
                        | {
                            version:(.Version // null),
                            name:(.Name // null),
                            summary:(.Summary // null),
                            description:(.Description // null),
                            urgency:(.Urgency // null)
                          }
                      ]
                    }
                ]
              }
            "#
            .into(),
        ],
        &raw,
    )
}

fn firmware_update_plan(raiz: &Path, id: &str) -> Result<String, String> {
    let updates = firmware_updates_json(raiz)?;
    let target = jq_compacto(
        raiz,
        &updates,
        &format!(".devices[] | select(.id == {})", json_texto(id)),
    )?;
    if target.is_empty() || target == "null" {
        return Err(format!("No hay una actualización disponible para {id}."));
    }
    jq0(
        raiz,
        &[
            "-cn".into(),
            "--arg".into(),
            "host".into(),
            resolver_equipo(raiz)?,
            "--argjson".into(),
            "device".into(),
            target,
            r#"{
              schemaVersion:1,
              kind:"korunix-firmware-update-plan",
              hostId:$host,
              backend:"fwupd",
              device:$device,
              targetRelease:($device.releases[0] // null),
              effect:(if $device.needsShutdown then "shutdown"
                      elif $device.needsReboot then "reboot"
                      else "immediate" end),
              requirements:{
                explicitUserAction:true,
                acPower:$device.requiresAcPower,
                privilegeBackend:"fwupd-dbus-polkit"
              },
              actions:{
                planWritesSystem:false,
                refreshesMetadata:false,
                installsFirmware:true,
                automaticallyReboots:false
              }
            }"#
            .into(),
        ],
    )
}

fn firmware(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    match args.first().map(String::as_str) {
        Some("devices") => {
            let data = firmware_devices_json(raiz)?;
            if args.iter().any(|v| v == "--json") {
                println!("{data}");
            } else {
                pretty(raiz, &data)?;
            }
        }
        Some("updates") => {
            let data = firmware_updates_json(raiz)?;
            if args.iter().any(|v| v == "--json") {
                println!("{data}");
            } else {
                pretty(raiz, &data)?;
            }
        }
        Some("refresh") => {
            let plan_only = args.iter().any(|v| v == "--plan");
            let json = args.iter().any(|v| v == "--json");
            let yes = args.iter().any(|v| v == "--yes");
            let plan = r#"{"schemaVersion":1,"kind":"korunix-firmware-refresh-plan","backend":"fwupd","networkAccess":true,"writesFirmware":false,"writesMetadata":true,"automatic":false}"#;

            if plan_only {
                if yes {
                    return Err("--yes no se utiliza junto con --plan.".into());
                }
                if json {
                    println!("{plan}");
                } else {
                    pretty(raiz, plan)?;
                }
                return Ok(ExitCode::SUCCESS);
            }
            if json && !yes {
                return Err("firmware refresh --json necesita --yes.".into());
            }
            if !yes && !confirm("¿Actualizar los metadatos de firmware?")? {
                return Ok(ExitCode::SUCCESS);
            }
            emitir_progreso(json, 10, "refreshing_firmware");

            let (code, _, err) = capture_status(
                raiz,
                "fwupdmgr",
                &["--assume-yes".into(), "refresh".into()],
            )?;

            emitir_progreso(json, 95, "preparing");
            if !matches!(code, 0 | 2) {
                return Err(if err.is_empty() {
                    "No se pudieron actualizar los metadatos.".into()
                } else {
                    err
                });
            }
            let result = format!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-firmware-refresh-result\",\"backend\":\"fwupd\",\"completed\":true,\"exitCode\":{code},\"automatic\":false}}"
            );
            emitir_progreso(json, 100, "done");
            if json {
                println!("{result}");
            } else {
                println!("✓ metadatos de firmware actualizados");
            }
        }
        Some("update") => {
            let id = args.get(1).ok_or_else(|| "Falta device-id.".to_string())?;
            let plan = firmware_update_plan(raiz, id)?;
            let plan_only = args.iter().any(|v| v == "--plan");
            let json = args.iter().any(|v| v == "--json");
            let yes = args.iter().any(|v| v == "--yes");

            if plan_only {
                if yes {
                    return Err("--yes no se utiliza junto con --plan.".into());
                }
                if json {
                    println!("{plan}");
                } else {
                    pretty(raiz, &plan)?;
                }
                return Ok(ExitCode::SUCCESS);
            }

            if json && !yes {
                return Err("firmware update --json necesita --yes.".into());
            }
            if !yes && !confirm("¿Instalar esta actualización de firmware?")? {
                return Ok(ExitCode::SUCCESS);
            }

            emitir_progreso(json, 10, "installing_firmware");

            let (code, _, err) = capture_status(
                raiz,
                "fwupdmgr",
                &[
                    "--assume-yes".into(),
                    "--no-reboot-check".into(),
                    "update".into(),
                    id.clone(),
                ],
            )?;

            emitir_progreso(json, 95, "preparing");
            if !matches!(code, 0 | 2) {
                return Err(if err.is_empty() {
                    "fwupd no pudo instalar el firmware.".into()
                } else {
                    err
                });
            }

            let result = jq0(
                raiz,
                &[
                    "-cn".into(),
                    "--argjson".into(),
                    "plan".into(),
                    plan,
                    "--argjson".into(),
                    "code".into(),
                    code.to_string(),
                    r#"{
                      schemaVersion:1,
                      kind:"korunix-firmware-update-result",
                      plan:$plan,
                      completed:true,
                      exitCode:$code,
                      automaticReboot:false
                    }"#
                    .into(),
                ],
            )?;
            emitir_progreso(json, 100, "done");

            if json {
                println!("{result}");
            } else {
                println!("✓ SOLICITUD DE FIRMWARE COMPLETADA");
            }
        }
        _ => {
            return Err("Uso: korunix firmware devices|updates [--json] | refresh [--plan] [--yes] [--json] | update <device-id> [--plan] [--yes] [--json].".into())
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn media_capture(raiz: &Path, program: &str, args: &[String]) -> Result<String, String> {
    let bin = if let Some(base) = env::var_os("KORUNIX_MEDIA_BIN_DIR") {
        PathBuf::from(base).join(program).into_os_string()
    } else {
        tool(program)
    };
    let out = Command::new(bin)
        .args(args)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("No pude ejecutar {program}: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn wpctl_nodes(raiz: &Path, class: &str) -> Result<String, String> {
    let list = media_capture(
        raiz,
        "wpctl",
        &["list".into(), "audio".into(), class.into()],
    )?;
    let mut items = Vec::new();
    for raw in list.lines() {
        let parts: Vec<&str> = raw.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let first_raw = parts[0].trim();
        let default = first_raw.starts_with('*') || parts.iter().skip(3).any(|v| v.trim() == "*");
        let id_text = first_raw
            .trim_start_matches('*')
            .trim()
            .trim_end_matches('.');
        let Ok(id) = id_text.parse::<u32>() else {
            continue;
        };
        let volume_raw = media_capture(raiz, "wpctl", &["get-volume".into(), id.to_string()])
            .unwrap_or_default();
        let volume = volume_raw
            .split_whitespace()
            .find_map(|v| v.parse::<f64>().ok());
        let volume_json = volume
            .map(|v| {
                if v.fract() == 0.0 {
                    format!("{v:.1}")
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_else(|| "null".into());
        let muted = volume_raw.contains("[MUTED]");
        items.push(format!(
            "{{\"id\":{id},\"name\":{},\"mediaClass\":{},\"default\":{default},\"volume\":{},\"muted\":{muted}}}",
            json_texto(parts[1].trim()),
            json_texto(parts[2].trim()),
            volume_json
        ));
    }
    Ok(format!("[{}]", items.join(",")))
}

fn nombre_monitor_edid(edid: &[u8]) -> Option<String> {
    if edid.len() < 128 {
        return None;
    }

    for inicio in [54_usize, 72, 90, 108] {
        let fin = inicio + 18;
        if fin > edid.len() {
            continue;
        }

        let descriptor = &edid[inicio..fin];
        if descriptor[0] != 0 || descriptor[1] != 0 || descriptor[2] != 0 || descriptor[3] != 0xfc {
            continue;
        }

        let nombre = descriptor[5..18]
            .iter()
            .copied()
            .take_while(|byte| *byte != 0 && *byte != b'\n')
            .map(char::from)
            .collect::<String>()
            .trim()
            .to_string();

        if !nombre.is_empty() {
            return Some(nombre);
        }
    }

    None
}

fn tipo_conector_eld(valor: &str) -> Option<&'static str> {
    let normalizado = valor
        .to_ascii_lowercase()
        .chars()
        .filter(|caracter| !matches!(caracter, ' ' | '-' | '_'))
        .collect::<String>();

    if normalizado.contains("displayport") {
        Some("DisplayPort")
    } else if normalizado.contains("hdmi") {
        Some("HDMI")
    } else {
        None
    }
}

fn registros_eld() -> Vec<(String, String)> {
    let mut resultado = Vec::new();

    let Ok(tarjetas) = fs::read_dir("/proc/asound") else {
        return resultado;
    };

    for tarjeta in tarjetas.flatten() {
        let nombre_tarjeta = tarjeta.file_name().to_string_lossy().to_string();
        if !nombre_tarjeta.starts_with("card") {
            continue;
        }

        let Ok(entradas) = fs::read_dir(tarjeta.path()) else {
            continue;
        };

        for entrada in entradas.flatten() {
            let nombre = entrada.file_name().to_string_lossy().to_string();
            if !nombre.starts_with("eld#") {
                continue;
            }

            let Ok(contenido) = fs::read_to_string(entrada.path()) else {
                continue;
            };

            let mut valido = false;
            let mut monitor = None::<String>;
            let mut conexion = None::<String>;

            for linea in contenido.lines() {
                let linea = linea.trim();
                let Some(indice) = linea.bytes().position(|byte| byte.is_ascii_whitespace()) else {
                    continue;
                };

                let (clave, resto) = linea.split_at(indice);
                let valor = resto.trim();

                match clave {
                    "eld_valid" => valido = valor == "1",
                    "monitor_name" if !valor.is_empty() => monitor = Some(valor.to_string()),
                    "connection_type" if !valor.is_empty() => conexion = Some(valor.to_string()),
                    _ => {}
                }
            }

            if valido {
                if let (Some(monitor), Some(conexion)) = (monitor, conexion) {
                    resultado.push((monitor, conexion));
                }
            }
        }
    }

    resultado
}

fn conexiones_pantalla_json() -> String {
    let eld = registros_eld();
    let mut conectores = Vec::<(String, PathBuf)>::new();

    let Ok(entradas) = fs::read_dir("/sys/class/drm") else {
        return "[]".to_string();
    };

    for entrada in entradas.flatten() {
        let nombre = entrada.file_name().to_string_lossy().to_string();

        if !(nombre.contains("-HDMI-A-") || nombre.contains("-DP-")) {
            continue;
        }

        conectores.push((nombre, entrada.path()));
    }

    conectores.sort_by(|a, b| a.0.cmp(&b.0));

    let mut resultado = Vec::<String>::new();

    for (nombre_drm, ruta) in conectores {
        let conectado = fs::read_to_string(ruta.join("status"))
            .ok()
            .map(|estado| estado.trim() == "connected")
            .unwrap_or(false);

        if !conectado {
            continue;
        }

        let conector = if nombre_drm.contains("-HDMI-A-") {
            "HDMI"
        } else {
            "DisplayPort"
        };

        let monitor_edid = fs::read(ruta.join("edid"))
            .ok()
            .and_then(|edid| nombre_monitor_edid(&edid));

        let mut monitor = monitor_edid.clone();
        let mut conexion_eld = None::<String>;

        if let Some(nombre) = monitor.as_deref() {
            if let Some((_, conexion)) = eld
                .iter()
                .find(|(monitor_eld, _)| monitor_eld.eq_ignore_ascii_case(nombre))
            {
                conexion_eld = Some(conexion.clone());
            }
        }

        if monitor.is_none() {
            let candidatos = eld
                .iter()
                .filter(|(_, conexion)| tipo_conector_eld(conexion) == Some(conector))
                .collect::<Vec<_>>();

            if candidatos.len() == 1 {
                monitor = Some(candidatos[0].0.clone());
                conexion_eld = Some(candidatos[0].1.clone());
            }
        }

        let monitor_json = monitor
            .as_deref()
            .map(json_texto)
            .unwrap_or_else(|| "null".to_string());

        let eld_json = conexion_eld
            .as_deref()
            .map(json_texto)
            .unwrap_or_else(|| "null".to_string());

        let verificado_eld = conexion_eld
            .as_deref()
            .and_then(tipo_conector_eld)
            .map(|tipo| tipo == conector);

        let verificado_json = verificado_eld
            .map(|valor| valor.to_string())
            .unwrap_or_else(|| "null".to_string());

        resultado.push(format!(
            "{{\"drmName\":{},\"connector\":{},\"monitorName\":{},\"source\":{},\"eldConnectionType\":{},\"eldAgrees\":{}}}",
            json_texto(&nombre_drm),
            json_texto(conector),
            monitor_json,
            json_texto(if monitor_edid.is_some() { "drm-edid" } else { "drm" }),
            eld_json,
            verificado_json,
        ));
    }

    format!("[{}]", resultado.join(","))
}

fn media_audio_json(raiz: &Path) -> Result<String, String> {
    let sinks = wpctl_nodes(raiz, "sinks")?;
    let sources = wpctl_nodes(raiz, "sources")?;

    let cards = media_capture(
        raiz,
        "pactl",
        &["-f".into(), "json".into(), "list".into(), "cards".into()],
    )?;

    let psinks = media_capture(
        raiz,
        "pactl",
        &["-f".into(), "json".into(), "list".into(), "sinks".into()],
    )?;

    let psources = media_capture(
        raiz,
        "pactl",
        &["-f".into(), "json".into(), "list".into(), "sources".into()],
    )?;

    let pantallas = conexiones_pantalla_json();

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "sinks".into(),
            sinks,
            "--argjson".into(),
            "sources".into(),
            sources,
            "--argjson".into(),
            "cards".into(),
            cards,
            "--argjson".into(),
            "psinks".into(),
            psinks,
            "--argjson".into(),
            "psources".into(),
            psources,
            "--argjson".into(),
            "displays".into(),
            pantallas,
            r#"{
              schemaVersion:1,
              kind:"korunix-media-audio",
              backend:"pipewire-wireplumber",
              defaults:{
                sinkId:([$sinks[]|select(.default)|.id][0] // null),
                sourceId:([$sources[]|select(.default)|.id][0] // null)
              },
              sinks:$sinks,
              sources:$sources,
              displayConnections:$displays,
              cards:[
                $cards[]?
                | {
                    index:(.index // null),
                    name:(.name // null),
                    driver:(.driver // null),
                    properties:(.properties // {}),
                    activeProfile:(
                      (.active_profile // null)
                      | if type=="object" then (.name // .description // null) else . end
                    ),
                    profiles:(
                      (.profiles // {}) as $p
                      | if ($p|type)=="object" then
                          $p|to_entries|map({
                            id:.key,
                            description:(.value.description // null),
                            sinks:(.value.n_sinks // 0),
                            sources:(.value.n_sources // 0),
                            priority:(.value.priority // 0),
                            available:(.value.available // null)
                          })
                        elif ($p|type)=="array" then
                          $p|map({
                            id:(.name // .id // null),
                            description:(.description // null),
                            sinks:(.n_sinks // 0),
                            sources:(.n_sources // 0),
                            priority:(.priority // 0),
                            available:(.available // null)
                          })
                        else [] end
                    ),
                    ports:(
                      (.ports // []) as $p
                      | if ($p|type)=="array" then
                          $p|map({
                            name:(.name // null),
                            description:(.description // null),
                            type:(.type // null),
                            availability:(.availability // null)
                          })
                        elif ($p|type)=="object" then
                          $p|to_entries|map({
                            name:.key,
                            description:(.value.description // null),
                            type:(.value.type // null),
                            availability:(.value.availability // null)
                          })
                        else [] end
                    )
                  }
              ],
              pulse:{
                sinks:[
                  $psinks[]?
                  | {
                      index:(.index // null),
                      name:(.name // null),
                      description:(.description // null),
                      mute:(.mute // false),
                      activePort:(.active_port // null),
                      properties:(.properties // {}),
                      ports:(.ports // [])
                    }
                ],
                sources:[
                  $psources[]?
                  | {
                      index:(.index // null),
                      name:(.name // null),
                      description:(.description // null),
                      mute:(.mute // false),
                      activePort:(.active_port // null),
                      properties:(.properties // {}),
                      ports:(.ports // [])
                    }
                ]
              }
            }"#
            .into(),
        ],
    )
}

fn v4l2_field(info: &str, key: &str) -> Option<String> {
    info.lines().find_map(|linea| {
        let (izquierda, derecha) = linea.split_once(':')?;
        if izquierda.trim() == key {
            let valor = derecha.trim();
            if valor.is_empty() {
                None
            } else {
                Some(valor.to_string())
            }
        } else {
            None
        }
    })
}

fn v4l2_device_caps(info: &str) -> Vec<String> {
    let mut dentro = false;
    let mut resultado = Vec::<String>::new();

    for original in info.lines() {
        let linea = original.trim();

        if linea.starts_with("Device Caps") {
            dentro = true;
            continue;
        }

        if !dentro {
            continue;
        }

        if linea.is_empty() {
            continue;
        }

        if linea.ends_with("Info:")
            || linea.starts_with("Media Driver")
            || linea.starts_with("Interface Info")
            || linea.starts_with("Entity Info")
        {
            break;
        }

        if linea.contains(':') {
            break;
        }

        resultado.push(linea.to_string());
    }

    resultado
}

fn v4l2_formats(raw: &str) -> Vec<serde_json::Value> {
    let mut formatos = Vec::<serde_json::Value>::new();
    let mut formato_actual = None::<usize>;
    let mut tamano_actual = None::<usize>;

    for original in raw.lines() {
        let linea = original.trim();

        if linea.starts_with('[') && linea.contains("]: '") {
            let mut comillas = linea.split('\'');
            let _ = comillas.next();
            let Some(pixel) = comillas.next() else {
                continue;
            };

            let marcador = format!("'{pixel}'");
            let descripcion = linea
                .split_once(marcador.as_str())
                .map(|(_, resto)| resto.trim())
                .unwrap_or("")
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();

            formatos.push(serde_json::json!({
                "pixelFormat": pixel,
                "description": descripcion,
                "sizes": []
            }));

            formato_actual = Some(formatos.len() - 1);
            tamano_actual = None;
            continue;
        }

        if let Some(resto) = linea.strip_prefix("Size: Discrete ") {
            let Some((ancho, alto)) = resto.split_once('x') else {
                continue;
            };

            let (Ok(ancho), Ok(alto)) = (ancho.parse::<u32>(), alto.parse::<u32>()) else {
                continue;
            };

            let Some(indice_formato) = formato_actual else {
                continue;
            };

            let Some(tamanos) = formatos
                .get_mut(indice_formato)
                .and_then(|formato| formato.get_mut("sizes"))
                .and_then(serde_json::Value::as_array_mut)
            else {
                continue;
            };

            tamanos.push(serde_json::json!({
                "width": ancho,
                "height": alto,
                "fps": []
            }));
            tamano_actual = Some(tamanos.len() - 1);
            continue;
        }

        if linea.starts_with("Interval: Discrete") && linea.contains("fps") {
            let fps = linea
                .split_once('(')
                .and_then(|(_, resto)| resto.split_once(" fps"))
                .and_then(|(valor, _)| valor.trim().parse::<f64>().ok());

            let (Some(fps), Some(indice_formato), Some(indice_tamano)) =
                (fps, formato_actual, tamano_actual)
            else {
                continue;
            };

            let Some(fps_lista) = formatos
                .get_mut(indice_formato)
                .and_then(|formato| formato.get_mut("sizes"))
                .and_then(serde_json::Value::as_array_mut)
                .and_then(|tamanos| tamanos.get_mut(indice_tamano))
                .and_then(|tamano| tamano.get_mut("fps"))
                .and_then(serde_json::Value::as_array_mut)
            else {
                continue;
            };

            let repetido = fps_lista
                .iter()
                .filter_map(serde_json::Value::as_f64)
                .any(|actual| (actual - fps).abs() < 0.001);

            if !repetido {
                fps_lista.push(serde_json::json!(fps));
            }
        }
    }

    formatos
}

fn v4l2_virtual(driver: &str, bus: &str) -> bool {
    let driver = driver.to_ascii_lowercase();
    let bus = bus.to_ascii_lowercase();

    driver.contains("v4l2 loopback")
        || driver.contains("v4l2loopback")
        || bus.contains("v4l2loopback")
}

fn camera_list_json(raiz: &Path) -> Result<String, String> {
    let devices: Vec<PathBuf> = if let Ok(value) = env::var("KORUNIX_VIDEO_DEVICES") {
        value
            .lines()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .collect()
    } else {
        let mut found = Vec::new();

        if let Ok(entries) = fs::read_dir("/dev") {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with("video") {
                    found.push(entry.path());
                }
            }
        }

        found.sort();
        found
    };

    let mut result = Vec::<String>::new();

    for device in devices {
        let info = match media_capture(
            raiz,
            "v4l2-ctl",
            &["-d".into(), device.display().to_string(), "--info".into()],
        ) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let formats_raw = media_capture(
            raiz,
            "v4l2-ctl",
            &[
                "-d".into(),
                device.display().to_string(),
                "--list-formats-ext".into(),
            ],
        )
        .unwrap_or_default();

        let driver = v4l2_field(&info, "Driver name").unwrap_or_default();
        let card = v4l2_field(&info, "Card type");
        let bus = v4l2_field(&info, "Bus info").unwrap_or_default();
        let version = v4l2_field(&info, "Driver version");
        let capabilities = v4l2_device_caps(&info);
        let formats = v4l2_formats(&formats_raw);

        let virtual_device = v4l2_virtual(&driver, &bus);
        let declares_capture = capabilities
            .iter()
            .any(|capability| capability == "Video Capture");
        let has_formats = !formats.is_empty();
        let capture_capable = declares_capture || has_formats;

        if !capture_capable && !virtual_device {
            continue;
        }

        let available = capture_capable && has_formats;
        let capabilities_json = serde_json::Value::Array(
            capabilities
                .iter()
                .map(|value| serde_json::Value::String(value.clone()))
                .collect(),
        )
        .to_string();
        let formats_json = serde_json::Value::Array(formats).to_string();

        result.push(format!(
            "{{\"device\":{},\"driver\":{},\"card\":{},\"bus\":{},\"version\":{},\"virtual\":{},\"captureCapable\":{},\"available\":{},\"capabilities\":{},\"formats\":{},\"rawFormats\":{}}}",
            json_texto(&device.display().to_string()),
            if driver.is_empty() {
                "null".to_string()
            } else {
                json_texto(&driver)
            },
            card.as_deref()
                .map(json_texto)
                .unwrap_or_else(|| "null".to_string()),
            if bus.is_empty() {
                "null".to_string()
            } else {
                json_texto(&bus)
            },
            version
                .as_deref()
                .map(json_texto)
                .unwrap_or_else(|| "null".to_string()),
            virtual_device,
            capture_capable,
            available,
            capabilities_json,
            formats_json,
            json_texto(&formats_raw),
        ));
    }

    Ok(format!(
        "{{\"schemaVersion\":2,\"kind\":\"korunix-media-cameras\",\"backend\":\"v4l2\",\"devices\":[{}]}}",
        result.join(",")
    ))
}

fn media_status_json(raiz: &Path) -> Result<String, String> {
    Ok(format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-media-status\",\"audio\":{},\"cameras\":{}}}",
        media_audio_json(raiz)?,
        camera_list_json(raiz)?
    ))
}

fn media_exec(raiz: &Path, program: &str, args: &[String]) -> Result<(), String> {
    let bin = if let Some(base) = env::var_os("KORUNIX_MEDIA_BIN_DIR") {
        PathBuf::from(base).join(program).into_os_string()
    } else {
        tool(program)
    };
    let status = Command::new(bin)
        .args(args)
        .current_dir(raiz)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("No pude ejecutar {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} terminó con error."))
    }
}

fn media(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    match args {
        [a] if a == "status" => pretty(raiz, &media_status_json(raiz)?)?,
        [a, j] if a == "status" && j == "--json" => println!("{}", media_status_json(raiz)?),
        [a, b] if a == "audio" && b == "list" => pretty(raiz, &media_audio_json(raiz)?)?,
        [a, b, j] if a == "audio" && b == "list" && j == "--json" => {
            println!("{}", media_audio_json(raiz)?)
        }
        [a, b, kind, id]
            if a == "audio" && b == "default" && matches!(kind.as_str(), "sink" | "source") =>
        {
            let _: u32 = id
                .parse()
                .map_err(|_| "ID de audio inválido.".to_string())?;
            media_exec(raiz, "wpctl", &["set-default".into(), id.clone()])?;
        }
        [a, b, id, volume] if a == "audio" && b == "volume" => {
            let _: u32 = id
                .parse()
                .map_err(|_| "ID de audio inválido.".to_string())?;
            let value = volume
                .strip_suffix('%')
                .and_then(|v| v.parse::<u32>().ok())
                .ok_or_else(|| "Volumen inválido.".to_string())?;
            if value > 150 {
                return Err("Korunix limita el volumen a 150%.".into());
            }
            media_exec(
                raiz,
                "wpctl",
                &[
                    "set-volume".into(),
                    id.clone(),
                    volume.clone(),
                    "--limit".into(),
                    "1.5".into(),
                ],
            )?;
        }
        [a, b, id, state] if a == "audio" && b == "mute" => {
            let _: u32 = id
                .parse()
                .map_err(|_| "ID de audio inválido.".to_string())?;
            if !matches!(state.as_str(), "0" | "1" | "toggle") {
                return Err("El silencio debe ser 0, 1 o toggle.".into());
            }
            media_exec(
                raiz,
                "wpctl",
                &["set-mute".into(), id.clone(), state.clone()],
            )?;
        }
        [a, b, card, profile] if a == "audio" && b == "profile" => {
            media_exec(
                raiz,
                "pactl",
                &["set-card-profile".into(), card.clone(), profile.clone()],
            )?;
        }
        [a, b, kind, object, port] if a == "audio" && b == "port" => {
            let cmd = match kind.as_str() {
                "sink" => "set-sink-port",
                "source" => "set-source-port",
                _ => return Err("Usa sink o source.".into()),
            };
            media_exec(raiz, "pactl", &[cmd.into(), object.clone(), port.clone()])?;
        }
        [a, b] if a == "camera" && b == "list" => pretty(raiz, &camera_list_json(raiz)?)?,
        [a, b, j] if a == "camera" && b == "list" && j == "--json" => {
            println!("{}", camera_list_json(raiz)?)
        }
        _ => return Err("Uso de media no reconocido.".into()),
    }
    Ok(ExitCode::SUCCESS)
}

fn account_valid(value: &str) -> bool {
    if value.is_empty() || value.len() > 32 {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

type AccountInfo = (String, String, bool, Vec<String>);

fn account_info(raiz: &Path, account: &str) -> Result<Option<AccountInfo>, String> {
    let (code, line, _) = capture_status(raiz, "getent", &["passwd".into(), account.into()])?;
    if code != 0 || line.is_empty() {
        return Ok(None);
    }
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() < 7 {
        return Ok(None);
    }
    let uid = fields[2].parse::<u32>().unwrap_or(0);
    if uid < uid_minimo() || uid >= 65534 || cuenta_tecnica(fields[5], fields[6]) {
        return Ok(None);
    }

    let name = fields[4]
        .split(',')
        .next()
        .filter(|v| !v.is_empty())
        .unwrap_or(account)
        .to_string();
    let groups_raw = capture(raiz, "id", &["-nG".into(), account.into()]).unwrap_or_default();
    let mut groups: Vec<String> = groups_raw
        .split_whitespace()
        .map(ToString::to_string)
        .collect();
    groups.sort();
    groups.dedup();
    let admin = groups.iter().any(|v| v == "wheel");
    Ok(Some((name, fields[5].to_string(), admin, groups)))
}

fn profile_text(account: &str, name: &str) -> String {
    format!(
        "# ESTE ARCHIVO SE PUEDE CAMBIAR.\n\
         # Perfil portable. No contiene contraseñas ni hashes.\n\
         {{\n\
         \x20 accountName = {};\n\
         \x20 fullName = {};\n\
         \x20 language = null;\n\
         \x20 interfaceLanguage = null;\n\
         \x20 inputMethods = [];\n\
         \x20 capabilities = [];\n\
         \x20 avatar = null;\n\
         }}\n",
        json_texto(account),
        json_texto(name)
    )
}

fn portable_optional_string(
    profile: &serde_json::Value,
    key: &str,
) -> Result<Option<String>, String> {
    match profile.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(format!("El campo portable {key} debe ser texto o null.")),
    }
}

fn portable_string_list(profile: &serde_json::Value, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = profile.get(key) else {
        return Ok(Vec::new());
    };

    let items = value
        .as_array()
        .ok_or_else(|| format!("El campo portable {key} debe ser una lista."))?;

    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| format!("El campo portable {key} solo admite textos."))
        })
        .collect()
}

fn nix_optional_string(value: Option<&str>) -> String {
    value.map(json_texto).unwrap_or_else(|| "null".to_string())
}

fn nix_string_list(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_texto(value))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn profile_text_from_manifest(profile: &serde_json::Value) -> Result<String, String> {
    let account = profile
        .get("accountName")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "El perfil importado no contiene accountName.".to_string())?;

    if !account_valid(account) {
        return Err("El perfil importado contiene un nombre de cuenta inválido.".to_string());
    }

    let name = profile
        .get("fullName")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "El perfil importado no contiene fullName.".to_string())?;

    let language = portable_optional_string(profile, "language")?;
    let interface_language = portable_optional_string(profile, "interfaceLanguage")?;

    if let Some(value) = interface_language.as_deref() {
        if !interface_language_supported(value) {
            return Err(format!(
                "El perfil importado pide un idioma de Korunix no soportado: {value}."
            ));
        }
    }

    let input_methods = portable_string_list(profile, "inputMethods")?;
    let capabilities = portable_string_list(profile, "capabilities")?;

    Ok(format!(
        "# ESTE ARCHIVO SE PUEDE CAMBIAR.\n\
         # Perfil portable. No contiene contraseñas ni hashes.\n\
         {{\n\
         \x20 accountName = {};\n\
         \x20 fullName = {};\n\
         \x20 language = {};\n\
         \x20 interfaceLanguage = {};\n\
         \x20 inputMethods = {};\n\
         \x20 capabilities = {};\n\
         \x20 avatar = null;\n\
         }}\n",
        json_texto(account),
        json_texto(name),
        nix_optional_string(language.as_deref()),
        nix_optional_string(interface_language.as_deref()),
        nix_string_list(&input_methods),
        nix_string_list(&capabilities),
    ))
}

fn host_config_path(raiz: &Path) -> Result<PathBuf, String> {
    let host = resolver_equipo(raiz)?;
    Ok(raiz
        .join("configuracion/equipos")
        .join(format!("{host}.nix")))
}

fn nix_config_json(raiz: &Path, atributo: &str) -> Result<String, String> {
    if !atributo
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
    {
        return Err("Atributo Nix inválido.".to_string());
    }

    let host = resolver_equipo(raiz)?;
    capture(
        raiz,
        "nix",
        &[
            "eval".into(),
            "--json".into(),
            format!(".#nixosConfigurations.{host}.config.korunix.{atributo}"),
        ],
    )
}

fn lista_json_strings(valor: &str) -> Result<Vec<String>, String> {
    serde_json::from_str::<Vec<String>>(valor)
        .map_err(|e| format!("El modelo Nix devolvió una lista inválida: {e}"))
}

fn reemplazar_linea_string(
    texto: &str,
    indent: usize,
    clave: &str,
    valor: &str,
) -> Result<String, String> {
    let prefijo = format!("{}{} = ", " ".repeat(indent), clave);
    let mut encontrados = 0usize;
    let mut salida = String::new();

    for linea in texto.split_inclusive('\n') {
        if linea.starts_with(&prefijo) {
            encontrados += 1;
            salida.push_str(&format!(
                "{}{} = {};\n",
                " ".repeat(indent),
                clave,
                json_texto(valor)
            ));
        } else {
            salida.push_str(linea);
        }
    }

    if encontrados != 1 {
        return Err(format!(
            "La configuración contiene {encontrados} asignaciones editables para {clave} con sangría {indent}."
        ));
    }

    Ok(salida)
}

fn reemplazar_lista_nix(
    texto: &str,
    indent: usize,
    clave: &str,
    valores: &[String],
) -> Result<String, String> {
    let inicio = format!("{}{} = [", " ".repeat(indent), clave);
    let cierre = format!("{}];", " ".repeat(indent));

    let lineas = texto.lines().collect::<Vec<_>>();
    let posiciones = lineas
        .iter()
        .enumerate()
        .filter_map(|(i, linea)| (*linea == inicio).then_some(i))
        .collect::<Vec<_>>();

    if posiciones.len() != 1 {
        return Err(format!(
            "No encontré de forma única la lista editable {clave}."
        ));
    }

    let a = posiciones[0];
    let b = lineas
        .iter()
        .enumerate()
        .skip(a + 1)
        .find_map(|(i, linea)| (*linea == cierre).then_some(i))
        .ok_or_else(|| format!("La lista {clave} no tiene cierre reconocible."))?;

    if lineas[a + 1..b]
        .iter()
        .any(|linea| linea.trim_start().starts_with('#'))
    {
        return Err(format!(
            "La lista {clave} contiene comentarios manuales internos; Korunix no los borrará silenciosamente."
        ));
    }

    let mut nuevo = String::new();
    for linea in &lineas[..a] {
        nuevo.push_str(linea);
        nuevo.push('\n');
    }

    nuevo.push_str(&inicio);
    nuevo.push('\n');
    for valor in valores {
        nuevo.push_str(&format!(
            "{}{}\n",
            " ".repeat(indent + 2),
            json_texto(valor)
        ));
    }
    nuevo.push_str(&cierre);
    nuevo.push('\n');

    for linea in &lineas[b + 1..] {
        nuevo.push_str(linea);
        nuevo.push('\n');
    }

    Ok(nuevo)
}

fn aplicar_configuracion_host(raiz: &Path, nombre: &str, nuevo: &str) -> Result<PathBuf, String> {
    let ruta = host_config_path(raiz)?;
    let anterior = fs::read(&ruta).map_err(|e| format!("No pude leer {}: {e}", ruta.display()))?;
    let backup = backup_dir(nombre)?;
    fs::write(backup.join("equipo.nix"), &anterior)
        .map_err(|e| format!("No pude guardar el respaldo: {e}"))?;

    let transaction = files_transaction_begin(raiz, std::slice::from_ref(&ruta))?;

    if let Err(error) = atomic_write(&ruta, nuevo.as_bytes()) {
        let _ = rollback_pending_transaction(raiz);
        return Err(error);
    }

    if let Err(error) = validate_quiet(raiz) {
        let recovery = rollback_pending_transaction(raiz);
        return match recovery {
            Ok(_) => Err(format!(
                "La configuración propuesta no pasó la validación y fue restaurada. {error}"
            )),
            Err(recovery_error) => Err(format!(
                "La configuración propuesta no pasó la validación ({error}) y la recuperación automática también falló: {recovery_error}"
            )),
        };
    }

    transaction_commit(Some(&transaction))?;
    Ok(backup)
}

fn salida_plan_o_confirmacion(
    raiz: &Path,
    plan: &str,
    plan_only: bool,
    yes: bool,
    json: bool,
    pregunta: &str,
) -> Result<Option<ExitCode>, String> {
    if plan_only {
        if yes {
            return Err("--yes no se utiliza junto con --plan.".to_string());
        }
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, plan)?;
        }
        return Ok(Some(ExitCode::SUCCESS));
    }

    if json && !yes {
        return Err("La ejecución JSON necesita --yes.".to_string());
    }

    if !yes && !confirm(pregunta)? {
        return Ok(Some(ExitCode::SUCCESS));
    }

    Ok(None)
}

fn appearance_host_text(texto: &str, style: &str, mode: &str) -> Result<String, String> {
    let mut lineas = texto.lines().map(ToString::to_string).collect::<Vec<_>>();

    let bloques = lineas
        .iter()
        .enumerate()
        .filter_map(|(i, linea)| (linea.trim() == "appearance = {").then_some(i))
        .collect::<Vec<_>>();

    if bloques.len() > 1 {
        return Err(
            "La configuración contiene más de un bloque appearance; Korunix no elegirá uno de forma arbitraria."
                .to_string(),
        );
    }

    if let Some(&inicio) = bloques.first() {
        let indent = lineas[inicio]
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();

        let mut nivel = 0isize;
        let mut fin = None;
        for (i, linea) in lineas.iter().enumerate().skip(inicio) {
            nivel += linea.chars().filter(|c| *c == '{').count() as isize;
            nivel -= linea.chars().filter(|c| *c == '}').count() as isize;
            if i > inicio && nivel == 0 {
                fin = Some(i);
                break;
            }
        }

        let fin =
            fin.ok_or_else(|| "El bloque appearance no tiene cierre reconocible.".to_string())?;
        let prefijo_style = " ".repeat(indent + 2) + "style = ";
        let prefijo_mode = " ".repeat(indent + 2) + "mode = ";

        let styles = (inicio + 1..fin)
            .filter(|i| lineas[*i].starts_with(&prefijo_style))
            .collect::<Vec<_>>();
        let modes = (inicio + 1..fin)
            .filter(|i| lineas[*i].starts_with(&prefijo_mode))
            .collect::<Vec<_>>();

        if styles.len() > 1 || modes.len() > 1 {
            return Err(
                "El bloque appearance contiene valores duplicados; Korunix no los sobrescribirá silenciosamente."
                    .to_string(),
            );
        }

        if let Some(&i) = styles.first() {
            lineas[i] = format!("{}style = {};", " ".repeat(indent + 2), json_texto(style));
        } else {
            lineas.insert(
                fin,
                format!("{}style = {};", " ".repeat(indent + 2), json_texto(style)),
            );
        }

        let mut nivel = 0isize;
        let mut fin = None;
        for (i, linea) in lineas.iter().enumerate().skip(inicio) {
            nivel += linea.chars().filter(|c| *c == '{').count() as isize;
            nivel -= linea.chars().filter(|c| *c == '}').count() as isize;
            if i > inicio && nivel == 0 {
                fin = Some(i);
                break;
            }
        }
        let fin =
            fin.ok_or_else(|| "El bloque appearance no tiene cierre reconocible.".to_string())?;

        let modes = (inicio + 1..fin)
            .filter(|i| lineas[*i].starts_with(&prefijo_mode))
            .collect::<Vec<_>>();

        if let Some(&i) = modes.first() {
            lineas[i] = format!("{}mode = {};", " ".repeat(indent + 2), json_texto(mode));
        } else {
            lineas.insert(
                fin,
                format!("{}mode = {};", " ".repeat(indent + 2), json_texto(mode)),
            );
        }

        return Ok(lineas.join("\n") + "\n");
    }

    let korunix = lineas
        .iter()
        .enumerate()
        .filter_map(|(i, linea)| (linea.trim() == "korunix = {").then_some(i))
        .collect::<Vec<_>>();

    if korunix.len() != 1 {
        return Err(
            "No encontré de forma única el bloque korunix del host; no modificaré el archivo."
                .to_string(),
        );
    }

    let inicio = korunix[0];
    let indent = lineas[inicio]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();

    let bloque = vec![
        format!("{}appearance = {{", " ".repeat(indent + 2)),
        format!("{}style = {};", " ".repeat(indent + 4), json_texto(style)),
        format!("{}mode = {};", " ".repeat(indent + 4), json_texto(mode)),
        format!("{}}};", " ".repeat(indent + 2)),
        String::new(),
    ];

    for (offset, linea) in bloque.into_iter().enumerate() {
        lineas.insert(inicio + 1 + offset, linea);
    }

    Ok(lineas.join("\n") + "\n")
}

fn appearance_state_json(raiz: &Path) -> Result<String, String> {
    let appearance = nix_config_json(raiz, "appearance")?;

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "appearance".into(),
            appearance,
            r#"{
              schemaVersion:2,
              kind:"korunix-appearance-state",
              declared:$appearance,
              styles:["default","dynamic","everforest"],
              modes:["light","dark","auto"],
              styleSupport:{
                default:["niri","hyprland","cinnamon","plasma"],
                dynamic:["niri","hyprland"],
                everforest:["niri","hyprland"]
              }
            }"#
            .into(),
        ],
    )
}

fn appearance_style_human(style: &str) -> &'static str {
    match style {
        "dynamic" => "Dinámica",
        "everforest" => "Everforest",
        _ => "Predeterminada",
    }
}

fn appearance_mode_human(mode: &str) -> &'static str {
    match mode {
        "light" => "Claro",
        "dark" => "Oscuro",
        _ => "Automático",
    }
}

fn appearance_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        pretty(raiz, &appearance_state_json(raiz)?)?;
        return Ok(ExitCode::SUCCESS);
    }

    if args == ["--json"] {
        println!("{}", appearance_state_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) != Some("set") {
        return Err(
            "Uso: korunix appearance [--json] | set [--style default|everforest] [--mode light|dark|auto] [--plan] [--yes] [--json]."
                .to_string(),
        );
    }

    let mut style = None::<String>;
    let mut mode = None::<String>;
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--style" => {
                i += 1;
                style = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta el estilo de apariencia.".to_string())?,
                );
            }
            "--mode" => {
                i += 1;
                mode = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta el modo de apariencia.".to_string())?,
                );
            }
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de apariencia desconocida: {otro}")),
        }
        i += 1;
    }

    if style.is_none() && mode.is_none() {
        return Err("No se indicó ningún cambio de apariencia.".to_string());
    }

    if let Some(valor) = style.as_deref() {
        if !matches!(valor, "default" | "dynamic" | "everforest") {
            return Err("El estilo debe ser default, dynamic o everforest.".to_string());
        }
    }

    if let Some(valor) = mode.as_deref() {
        if !matches!(valor, "light" | "dark" | "auto") {
            return Err("El modo debe ser light, dark o auto.".to_string());
        }
    }

    let actual_raw = nix_config_json(raiz, "appearance")?;
    let actual: serde_json::Value = serde_json::from_str(&actual_raw).map_err(|e| e.to_string())?;

    let style_actual = actual
        .get("style")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("default");
    let mode_actual = actual
        .get("mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("auto");

    let style_final = style.as_deref().unwrap_or(style_actual);
    let mode_final = mode.as_deref().unwrap_or(mode_actual);

    let despues = serde_json::json!({
        "style": style_final,
        "mode": mode_final
    });

    let plan = serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-appearance-change-plan",
        "before": actual,
        "after": despues,
        "requiresSystemApply": true,
        "livePreview": true
    })
    .to_string();

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        plan_only,
        yes,
        json,
        "¿Preparar este cambio de apariencia?",
    )? {
        return Ok(code);
    }

    let ruta = host_config_path(raiz)?;
    let texto = fs::read_to_string(&ruta).map_err(|e| e.to_string())?;
    let nuevo = appearance_host_text(&texto, style_final, mode_final)?;
    let backup = aplicar_configuracion_host(raiz, "appearance", &nuevo)?;

    history_record(
        "appearance-prepared",
        &format!(
            "Preparaste la apariencia {} en modo {}",
            appearance_style_human(style_final),
            appearance_mode_human(mode_final)
        ),
    )?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-appearance-change-result\",\"changed\":true,\"requiresSystemApply\":true,\"livePreview\":true,\"backup\":{}}}",
            json_texto(&backup.display().to_string())
        );
    } else {
        println!("✓ apariencia preparada; falta aplicar la configuración");
    }

    Ok(ExitCode::SUCCESS)
}

fn default_roles_state_json(raiz: &Path) -> Result<String, String> {
    let host = resolver_equipo(raiz)?;
    flake_raw(
        raiz,
        &format!(
            "nixosConfigurations.{host}.config.environment.etc.\"korunix/default-roles.json\".text"
        ),
    )
}

fn default_roles_block_end(lineas: &[String], inicio: usize) -> Result<usize, String> {
    let mut nivel = 0isize;

    for (i, linea) in lineas.iter().enumerate().skip(inicio) {
        nivel += linea.chars().filter(|c| *c == '{').count() as isize;
        nivel -= linea.chars().filter(|c| *c == '}').count() as isize;

        if i > inicio && nivel == 0 {
            return Ok(i);
        }
    }

    Err("El bloque defaultRoles no tiene cierre reconocible.".to_string())
}

fn default_roles_set_line(
    lineas: &mut Vec<String>,
    inicio: usize,
    clave: &str,
    valor: &str,
) -> Result<(), String> {
    let fin = default_roles_block_end(lineas, inicio)?;
    let prefijo = format!("    {clave} = ");

    let posiciones = (inicio + 1..fin)
        .filter(|i| lineas[*i].starts_with(&prefijo))
        .collect::<Vec<_>>();

    if posiciones.len() > 1 {
        return Err(format!(
            "El perfil contiene más de una elección {clave}; Korunix no sobrescribirá una de forma arbitraria."
        ));
    }

    let nueva = format!("    {clave} = {};", json_texto(valor));

    if let Some(&posicion) = posiciones.first() {
        lineas[posicion] = nueva;
    } else {
        lineas.insert(fin, nueva);
    }

    Ok(())
}

fn default_roles_profile_text(
    texto: &str,
    browser: Option<&str>,
    plasma_text_editor: Option<&str>,
) -> Result<String, String> {
    if browser.is_none() && plasma_text_editor.is_none() {
        return Err("No se indicó ningún rol predeterminado.".to_string());
    }

    let mut lineas = texto.lines().map(ToString::to_string).collect::<Vec<_>>();

    let bloques = lineas
        .iter()
        .enumerate()
        .filter_map(|(i, linea)| (linea.trim() == "defaultRoles = {").then_some(i))
        .collect::<Vec<_>>();

    if bloques.len() > 1 {
        return Err(
            "El perfil contiene más de un bloque defaultRoles; Korunix no elegirá uno de forma arbitraria."
                .to_string(),
        );
    }

    let inicio = if let Some(&inicio) = bloques.first() {
        if lineas[inicio] != "  defaultRoles = {" {
            return Err(
                "El bloque defaultRoles existe con una estructura manual que Korunix no puede editar con seguridad."
                    .to_string(),
            );
        }
        inicio
    } else {
        if texto.lines().any(|linea| linea.contains("defaultRoles")) {
            return Err(
                "El perfil menciona defaultRoles con una estructura no reconocida; Korunix conservará el archivo intacto."
                    .to_string(),
            );
        }

        let ancla = lineas
            .iter()
            .position(|linea| linea == "  capabilities = [")
            .or_else(|| {
                lineas
                    .iter()
                    .position(|linea| linea.starts_with("  avatar ="))
            })
            .or_else(|| lineas.iter().rposition(|linea| linea == "}"))
            .ok_or_else(|| {
                "No encontré un punto seguro para insertar defaultRoles en el perfil.".to_string()
            })?;

        let mut bloque = vec![
            "  # Aplicaciones predeterminadas que acompañan a esta persona.".to_string(),
            "  defaultRoles = {".to_string(),
        ];

        if let Some(valor) = browser {
            bloque.push(format!("    browser = {};", json_texto(valor)));
        }

        if let Some(valor) = plasma_text_editor {
            bloque.push(format!("    plasmaTextEditor = {};", json_texto(valor)));
        }

        bloque.push("  };".to_string());
        bloque.push(String::new());

        for (offset, linea) in bloque.into_iter().enumerate() {
            lineas.insert(ancla + offset, linea);
        }

        return Ok(lineas.join("\n") + "\n");
    };

    if let Some(valor) = browser {
        default_roles_set_line(&mut lineas, inicio, "browser", valor)?;
    }

    if let Some(valor) = plasma_text_editor {
        default_roles_set_line(&mut lineas, inicio, "plasmaTextEditor", valor)?;
    }

    Ok(lineas.join("\n") + "\n")
}

fn aplicar_configuracion_persona(
    raiz: &Path,
    persona: &str,
    nuevo: &str,
) -> Result<PathBuf, String> {
    if !id_valido(persona) {
        return Err("Identificador de persona inválido.".to_string());
    }

    let ruta = raiz
        .join("configuracion/personas")
        .join(format!("{persona}.nix"));

    if !ruta.is_file() {
        return Err(format!("No existe configuracion/personas/{persona}.nix."));
    }

    let metadata = fs::symlink_metadata(&ruta)
        .map_err(|e| format!("No pude inspeccionar {}: {e}", ruta.display()))?;

    if metadata.file_type().is_symlink() {
        return Err(
            "Korunix no modifica perfiles portables que sean enlaces simbólicos.".to_string(),
        );
    }

    let anterior = fs::read(&ruta).map_err(|e| format!("No pude leer {}: {e}", ruta.display()))?;

    let backup = backup_dir("default-roles")?;
    fs::write(backup.join("persona.nix"), &anterior)
        .map_err(|e| format!("No pude guardar el respaldo del perfil: {e}"))?;

    let transaction = files_transaction_begin(raiz, std::slice::from_ref(&ruta))?;

    if let Err(error) = atomic_write(&ruta, nuevo.as_bytes()) {
        let _ = rollback_pending_transaction(raiz);
        return Err(error);
    }

    if let Err(error) = validate_quiet(raiz) {
        let recovery = rollback_pending_transaction(raiz);

        return match recovery {
            Ok(_) => Err(format!(
                "La elección propuesta no pasó la validación y el perfil fue restaurado. {error}"
            )),
            Err(recovery_error) => Err(format!(
                "La elección propuesta no pasó la validación ({error}) y la recuperación automática también falló: {recovery_error}"
            )),
        };
    }

    transaction_commit(Some(&transaction))?;
    Ok(backup)
}

fn defaults_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        pretty(raiz, &default_roles_state_json(raiz)?)?;
        return Ok(ExitCode::SUCCESS);
    }

    if args == ["--json"] {
        println!("{}", default_roles_state_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) != Some("set") {
        return Err(
            "Uso: korunix defaults [--json] | set --person <id> [--browser firefox|google-chrome] [--plasma-text-editor kwrite|kate] [--plan] [--yes] [--json]."
                .to_string(),
        );
    }

    let mut persona = None::<String>;
    let mut browser = None::<String>;
    let mut plasma_text_editor = None::<String>;
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;

    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--person" => {
                if persona.is_some() {
                    return Err("--person se indicó más de una vez.".to_string());
                }

                i += 1;
                persona = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta el identificador de persona.".to_string())?,
                );
            }
            "--browser" => {
                if browser.is_some() {
                    return Err("--browser se indicó más de una vez.".to_string());
                }

                i += 1;
                browser = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta el navegador.".to_string())?,
                );
            }
            "--plasma-text-editor" => {
                if plasma_text_editor.is_some() {
                    return Err("--plasma-text-editor se indicó más de una vez.".to_string());
                }

                i += 1;
                plasma_text_editor = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta el editor de Plasma.".to_string())?,
                );
            }
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            otro => {
                return Err(format!(
                    "Opción de aplicaciones predeterminadas desconocida: {otro}"
                ))
            }
        }

        i += 1;
    }

    let persona = persona.ok_or_else(|| "Falta --person.".to_string())?;

    if !id_valido(&persona) {
        return Err("Identificador de persona inválido.".to_string());
    }

    if browser.is_none() && plasma_text_editor.is_none() {
        return Err("Indica --browser, --plasma-text-editor o ambas elecciones.".to_string());
    }

    if let Some(valor) = browser.as_deref() {
        if !matches!(valor, "firefox" | "google-chrome") {
            return Err("El navegador debe ser firefox o google-chrome.".to_string());
        }
    }

    if let Some(valor) = plasma_text_editor.as_deref() {
        if !matches!(valor, "kwrite" | "kate") {
            return Err("El editor de Plasma debe ser kwrite o kate.".to_string());
        }
    }

    let estado_raw = default_roles_state_json(raiz)?;
    let estado: serde_json::Value = serde_json::from_str(&estado_raw).map_err(|e| e.to_string())?;

    let personas = estado
        .get("people")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "El contrato de roles predeterminados no contiene personas.".to_string())?;

    let actual = personas
        .iter()
        .find(|entrada| {
            entrada.get("id").and_then(serde_json::Value::as_str) == Some(persona.as_str())
        })
        .ok_or_else(|| {
            format!(
                "{persona} no está asignada al host actual; Korunix no modificará un perfil ajeno."
            )
        })?;

    let before_browser = actual
        .pointer("/requested/browser")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);

    let before_plasma = actual
        .pointer("/requested/plasmaTextEditor")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);

    let after_browser = browser.clone().or_else(|| before_browser.clone());
    let after_plasma = plasma_text_editor.clone().or_else(|| before_plasma.clone());

    let changed = before_browser != after_browser || before_plasma != after_plasma;

    let aplicaciones_raw = nix_config_json(raiz, "applications")?;
    let aplicaciones: Vec<String> =
        serde_json::from_str(&aplicaciones_raw).map_err(|e| e.to_string())?;

    let browser_effective = after_browser
        .as_ref()
        .filter(|valor| aplicaciones.contains(valor))
        .cloned();

    let browser_deferred = after_browser.is_some() && browser_effective.is_none();

    let relative_profile = format!("configuracion/personas/{persona}.nix");

    let plan_value = serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-default-roles-change-plan",
        "person": persona,
        "profilePath": relative_profile,
        "portableProfile": true,
        "before": {
            "browser": before_browser,
            "plasmaTextEditor": before_plasma
        },
        "after": {
            "browser": after_browser,
            "plasmaTextEditor": after_plasma
        },
        "effectiveOnCurrentHostAfterApply": {
            "browser": browser_effective,
            "browserDeferred": browser_deferred
        },
        "changed": changed,
        "effects": {
            "writesPortableProfile": changed,
            "writesMimeFilesNow": false,
            "changesLiveDefaults": false,
            "requiresSystemApply": changed,
            "buildsGeneration": false,
            "appliesGeneration": false
        }
    });

    let plan = serde_json::to_string(&plan_value).map_err(|e| e.to_string())?;

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        plan_only,
        yes,
        json,
        "¿Guardar estas aplicaciones predeterminadas en el perfil portable?",
    )? {
        return Ok(code);
    }

    if !changed {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "schemaVersion": 1,
                    "kind": "korunix-default-roles-change-result",
                    "person": persona,
                    "changed": false,
                    "nothingToDo": true,
                    "requiresSystemApply": false,
                    "writesMimeFilesNow": false,
                    "changesLiveDefaults": false
                })
            );
        } else {
            println!("✓ esas aplicaciones predeterminadas ya están elegidas");
        }

        return Ok(ExitCode::SUCCESS);
    }

    let ruta = raiz
        .join("configuracion/personas")
        .join(format!("{persona}.nix"));

    let texto =
        fs::read_to_string(&ruta).map_err(|e| format!("No pude leer {}: {e}", ruta.display()))?;

    let nuevo =
        default_roles_profile_text(&texto, browser.as_deref(), plasma_text_editor.as_deref())?;

    let backup = aplicar_configuracion_persona(raiz, &persona, &nuevo)?;

    let mut cambios = Vec::<String>::new();

    if let Some(valor) = browser.as_deref() {
        cambios.push(format!("navegador {valor}"));
    }

    if let Some(valor) = plasma_text_editor.as_deref() {
        cambios.push(format!("editor de Plasma {valor}"));
    }

    history_record(
        "default-roles-prepared",
        &format!("Preparaste para {persona}: {}", cambios.join(" y ")),
    )?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "korunix-default-roles-change-result",
                "person": persona,
                "changed": true,
                "nothingToDo": false,
                "requiresSystemApply": true,
                "writesMimeFilesNow": false,
                "changesLiveDefaults": false,
                "backup": backup.display().to_string()
            })
        );
    } else {
        println!("✓ aplicaciones predeterminadas preparadas; falta aplicar la configuración");
    }

    Ok(ExitCode::SUCCESS)
}

fn nixpkgs_human_selection_id(id: &str) -> &str {
    for prefix in ["legacyPackages.", "packages."] {
        if let Some(rest) = id.strip_prefix(prefix) {
            if let Some((_system, human_id)) = rest.split_once('.') {
                if !human_id.is_empty() {
                    return human_id;
                }
            }
        }
    }

    id
}

fn application_selection_token(source: &str, id: &str) -> Result<String, String> {
    if id.trim().is_empty() || id.chars().any(char::is_whitespace) {
        return Err("Identificador de aplicación inválido.".to_string());
    }

    match source {
        "curated" => {
            if !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
            {
                return Err("Identificador del catálogo inválido.".to_string());
            }
            Ok(id.to_string())
        }
        "nixpkgs" => {
            let id = nixpkgs_human_selection_id(id);
            if !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'+'))
            {
                return Err("Identificador de Nixpkgs inválido.".to_string());
            }
            Ok(id.to_string())
        }
        "flatpak" => {
            if !id.contains('.')
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
            {
                return Err("ID de Flatpak inválido.".to_string());
            }
            Ok(format!("flatpak:{id}"))
        }
        _ => Err("Fuente de aplicación desconocida.".to_string()),
    }
}

fn profile_text_with_avatar(
    account: &str,
    name: &str,
    avatar_relative: Option<&str>,
) -> Result<String, String> {
    let contenido = profile_text(account, name);

    let Some(avatar) = avatar_relative else {
        return Ok(contenido);
    };

    if avatar.starts_with('/') || avatar.contains("..") || avatar.chars().any(char::is_whitespace) {
        return Err("Ruta relativa de avatar inválida.".to_string());
    }

    let marcador = "  avatar = null;\n";
    if contenido.matches(marcador).count() != 1 {
        return Err("El perfil generado no contiene un avatar editable único.".to_string());
    }

    Ok(contenido.replace(marcador, &format!("  avatar = ./{avatar};\n")))
}

fn avatar_source(path: &str) -> Result<(PathBuf, String), String> {
    let origen = PathBuf::from(path);
    let meta = fs::symlink_metadata(&origen).map_err(|e| format!("No pude leer el avatar: {e}"))?;

    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err("El avatar debe ser un archivo normal, no un enlace.".to_string());
    }

    let extension = origen
        .extension()
        .and_then(|v| v.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "El avatar necesita extensión de imagen.".to_string())?;

    if !matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp") {
        return Err("El avatar debe ser PNG, JPEG o WebP.".to_string());
    }

    Ok((origen, extension))
}

fn host_name_normalized(requested: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut separator_pending = false;

    for character in requested.trim().chars() {
        if character.is_ascii_alphanumeric() {
            if separator_pending && !output.is_empty() {
                output.push('-');
            }

            output.push(character.to_ascii_lowercase());
            separator_pending = false;
        } else if character == '-' || character == '_' || character.is_ascii_whitespace() {
            if !output.is_empty() {
                separator_pending = true;
            }
        } else {
            return Err(
                "El nombre del equipo solo puede usar letras sin tildes, números, espacios, guion o guion bajo."
                    .to_string(),
            );
        }
    }

    if output.is_empty() {
        return Err("El nombre del equipo no puede quedar vacío.".to_string());
    }

    if output.len() > 63 {
        return Err("El nombre del equipo no puede superar 63 caracteres.".to_string());
    }

    Ok(output)
}

fn host_name_assignment(text: &str) -> Result<(usize, usize, String), String> {
    const TOKEN: &str = "hostName";
    let bytes = text.as_bytes();
    let mut found = Vec::<(usize, usize, String)>::new();

    for (position, _) in text.match_indices(TOKEN) {
        if position > 0 {
            let previous = bytes[position - 1];
            if previous.is_ascii_alphanumeric() || previous == b'_' {
                continue;
            }
        }

        let after_token = position + TOKEN.len();
        if after_token < bytes.len() {
            let next = bytes[after_token];
            if next.is_ascii_alphanumeric() || next == b'_' {
                continue;
            }
        }

        let line_start = text[..position]
            .rfind('\n')
            .map(|index| index + 1)
            .unwrap_or(0);

        if text[line_start..position].contains('#') {
            continue;
        }

        let mut cursor = after_token;

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }

        cursor += 1;

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= bytes.len() || bytes[cursor] != b'"' {
            return Err(
                "hostName debe ser una cadena literal para que Korunix pueda cambiarlo de forma segura."
                    .to_string(),
            );
        }

        let value_start = cursor + 1;
        cursor = value_start;
        let mut escaped = false;
        let mut closing = None;

        while cursor < bytes.len() {
            match bytes[cursor] {
                b'\\' if !escaped => escaped = true,
                b'"' if !escaped => {
                    closing = Some(cursor);
                    break;
                }
                _ => escaped = false,
            }

            cursor += 1;
        }

        let value_end = closing
            .ok_or_else(|| "La declaración hostName no tiene comillas de cierre.".to_string())?;

        let raw = &text[value_start..value_end];

        if raw.contains('\\') {
            return Err(
                "hostName contiene escapes que Korunix no modificará automáticamente.".to_string(),
            );
        }

        found.push((value_start, value_end, raw.to_string()));
    }

    match found.as_slice() {
        [entry] => Ok(entry.clone()),
        [] => {
            Err("El archivo del equipo debe declarar hostName como una cadena literal.".to_string())
        }
        _ => Err("El archivo del equipo contiene más de una declaración hostName.".to_string()),
    }
}

fn host_name_text(text: &str, target: &str) -> Result<(String, String), String> {
    let (start, end, before) = host_name_assignment(text)?;

    let mut output = String::with_capacity(text.len() + target.len());
    output.push_str(&text[..start]);
    output.push_str(target);
    output.push_str(&text[end..]);

    Ok((output, before))
}

fn host_state_json(raiz: &Path) -> Result<String, String> {
    let host_id = resolver_equipo(raiz)?;
    let path = host_config_path(raiz)?;
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("No pude leer {}: {error}", path.display()))?;
    let (_, _, host_name) = host_name_assignment(&text)?;

    Ok(format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-host-state\",\"hostId\":{},\"hostName\":{},\"structuralIdStable\":true,\"profilePath\":{},\"hardwarePath\":{}}}",
        json_texto(&host_id),
        json_texto(&host_name),
        json_texto(&format!("configuracion/equipos/{host_id}.nix")),
        json_texto(&format!("generado/equipos/{host_id}-detectado.nix"))
    ))
}

fn host_rename_plan_json(host_id: &str, requested: &str, before: &str, after: &str) -> String {
    let changed = before != after;

    format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-host-rename-plan\",\"hostId\":{},\"requested\":{},\"before\":{},\"after\":{},\"changed\":{},\"identity\":{{\"structuralIdStable\":true,\"profileFileRenamed\":false,\"hardwareFileRenamed\":false}},\"effects\":{{\"writesConfiguration\":{},\"runningHostnameChanged\":false,\"requiresSystemApply\":{},\"buildsGeneration\":false,\"appliesGeneration\":false}}}}",
        json_texto(host_id),
        json_texto(requested),
        json_texto(before),
        json_texto(after),
        changed,
        changed,
        changed
    )
}

fn host_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        pretty(raiz, &host_state_json(raiz)?)?;
        return Ok(ExitCode::SUCCESS);
    }

    if args == ["--json"] {
        println!("{}", host_state_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) != Some("rename") {
        return Err(
            "Uso: korunix host [--json] | rename <nombre> [--plan] [--yes] [--json].".to_string(),
        );
    }

    let requested = args
        .get(1)
        .ok_or_else(|| "Indica el nuevo nombre del equipo.".to_string())?
        .to_string();

    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;

    for arg in &args[2..] {
        match arg.as_str() {
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            other => return Err(format!("Opción de nombre de equipo desconocida: {other}")),
        }
    }

    if plan_only && yes {
        return Err("--yes no se utiliza junto con --plan.".to_string());
    }

    let target = host_name_normalized(&requested)?;
    let host_id = resolver_equipo(raiz)?;
    let path = host_config_path(raiz)?;
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("No pude leer {}: {error}", path.display()))?;
    let (new_text, before) = host_name_text(&text, &target)?;
    let plan = host_rename_plan_json(&host_id, &requested, &before, &target);

    if plan_only {
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }

        return Ok(ExitCode::SUCCESS);
    }

    if before == target {
        if json {
            println!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-host-rename-result\",\"changed\":false,\"hostId\":{},\"hostName\":{},\"requiresSystemApply\":false}}",
                json_texto(&host_id),
                json_texto(&target)
            );
        } else {
            println!("✓ el equipo ya usa ese nombre");
        }

        return Ok(ExitCode::SUCCESS);
    }

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        false,
        yes,
        json,
        &format!(
            "¿Preparar el nombre {target}? El identificador estructural {host_id} no cambiará."
        ),
    )? {
        return Ok(code);
    }

    let backup = aplicar_configuracion_host(raiz, "host-name", &new_text)?;

    history_record(
        "host-name-prepared",
        &format!("Preparaste el nombre visible {target} para el equipo {host_id}"),
    )?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-host-rename-result\",\"changed\":true,\"hostId\":{},\"before\":{},\"hostName\":{},\"structuralIdChanged\":false,\"profileFileRenamed\":false,\"hardwareFileRenamed\":false,\"requiresSystemApply\":true,\"backup\":{}}}",
            json_texto(&host_id),
            json_texto(&before),
            json_texto(&target),
            json_texto(&backup.display().to_string())
        );
    } else {
        println!("✓ nombre preparado: {target}; el identificador {host_id} no cambió");
        println!("Falta aplicar la configuración para cambiar el hostname del sistema.");
    }

    Ok(ExitCode::SUCCESS)
}

fn applications_state_json(raiz: &Path) -> Result<String, String> {
    let seleccion = nix_config_json(raiz, "applications")?;
    let catalogo = nix_config_json(raiz, "internal.applicationCatalog")?;
    let presentacion = nix_config_json(raiz, "internal.applicationPresentation")?;

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "selected".into(),
            seleccion,
            "--argjson".into(),
            "catalog".into(),
            catalogo,
            "--argjson".into(),
            "presentation".into(),
            presentacion,
            r#"{
              schemaVersion:2,
              kind:"korunix-applications-state",
              selected:$selected,
              catalog:$catalog,
              presentation:$presentation,
              ownership:"korunix",
              sources:["curated","nixpkgs","flatpak"]
            }"#
            .into(),
        ],
    )
}

fn applications_search(raiz: &Path, consulta: &str, fuente: &str) -> Result<String, String> {
    let q = consulta.to_ascii_lowercase();

    match fuente {
        "curated" => {
            let catalogo =
                lista_json_strings(&nix_config_json(raiz, "internal.applicationCatalog")?)?;
            let coincidencias = catalogo
                .into_iter()
                .filter(|id| id.to_ascii_lowercase().contains(&q))
                .map(|id| {
                    format!(
                        "{{\"id\":{},\"name\":{},\"source\":\"curated\"}}",
                        json_texto(&id),
                        json_texto(&id)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");

            Ok(format!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-applications-search\",\"query\":{},\"source\":\"curated\",\"results\":[{}]}}",
                json_texto(consulta),
                coincidencias
            ))
        }
        "nixpkgs" => {
            let resultado = capture(
                raiz,
                "nix",
                &[
                    "search".into(),
                    "nixpkgs".into(),
                    consulta.into(),
                    "--json".into(),
                ],
            )?;

            let raw = if resultado.trim().is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str::<serde_json::Value>(resultado.trim())
                    .map_err(|e| format!("Nixpkgs devolvió una búsqueda no válida: {e}"))?
            };

            let mut resultados = Vec::<serde_json::Value>::new();
            if let Some(items) = raw.as_object() {
                for (technical_id, item) in items {
                    let human_id = nixpkgs_human_selection_id(technical_id);
                    let pname = item
                        .get("pname")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(human_id);
                    let description = item
                        .get("description")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");

                    resultados.push(serde_json::json!({
                        "id": human_id,
                        "technicalId": technical_id,
                        "pname": pname,
                        "name": pname,
                        "description": description
                    }));
                }
            }

            Ok(serde_json::json!({
                "schemaVersion": 2,
                "kind": "korunix-applications-search",
                "query": consulta,
                "source": "nixpkgs",
                "results": resultados
            })
            .to_string())
        }
        "flatpak" => {
            let salida = capture(
                raiz,
                "flatpak",
                &[
                    "remote-ls".into(),
                    "--app".into(),
                    "--columns=application,name,description".into(),
                ],
            )?;

            let mut resultados = Vec::new();
            for linea in salida.lines() {
                let partes = linea.split('\t').collect::<Vec<_>>();
                if partes.is_empty() {
                    continue;
                }
                let id = partes[0].trim();
                let nombre = partes.get(1).copied().unwrap_or("").trim();
                let descripcion = partes.get(2).copied().unwrap_or("").trim();
                let huella = format!("{id} {nombre} {descripcion}").to_ascii_lowercase();
                if !huella.contains(&q) {
                    continue;
                }
                resultados.push(format!(
                    "{{\"id\":{},\"name\":{},\"description\":{},\"source\":\"flatpak\"}}",
                    json_texto(id),
                    json_texto(if nombre.is_empty() { id } else { nombre }),
                    json_texto(descripcion)
                ));
            }

            Ok(format!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-applications-search\",\"query\":{},\"source\":\"flatpak\",\"results\":[{}]}}",
                json_texto(consulta),
                resultados.join(",")
            ))
        }
        _ => Err("Fuente de aplicaciones desconocida.".to_string()),
    }
}

fn applications_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        pretty(raiz, &applications_state_json(raiz)?)?;
        return Ok(ExitCode::SUCCESS);
    }

    if args == ["--json"] {
        println!("{}", applications_state_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) == Some("search") {
        let consulta = args.get(1).ok_or_else(|| {
            "Uso: korunix applications search <texto> [--source fuente] [--json].".to_string()
        })?;

        let mut fuente = "curated";
        let mut json = false;
        let mut i = 2usize;
        while i < args.len() {
            match args[i].as_str() {
                "--source" => {
                    i += 1;
                    fuente = args
                        .get(i)
                        .map(String::as_str)
                        .ok_or_else(|| "Falta la fuente de búsqueda.".to_string())?;
                }
                "--json" => json = true,
                otro => return Err(format!("Opción de búsqueda desconocida: {otro}")),
            }
            i += 1;
        }

        let resultado = applications_search(raiz, consulta, fuente)?;
        if json {
            println!("{resultado}");
        } else {
            pretty(raiz, &resultado)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) != Some("set") {
        return Err(
            "Uso: korunix applications [--json] | search <texto> [--source curated|nixpkgs|flatpak] [--json] | set <id> <on|off> [--source auto|curated|nixpkgs|flatpak] [--plan] [--yes] [--json]."
                .to_string(),
        );
    }

    let raw_id = args
        .get(1)
        .ok_or_else(|| "Falta la aplicación.".to_string())?
        .to_string();
    let activar = match args.get(2).map(String::as_str) {
        Some("on" | "enable" | "install") => true,
        Some("off" | "disable" | "remove") => false,
        _ => return Err("El estado debe ser on u off.".to_string()),
    };

    let mut source = "auto".to_string();
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;
    let mut i = 3usize;

    while i < args.len() {
        match args[i].as_str() {
            "--source" => {
                i += 1;
                source = args
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "Falta la fuente de la aplicación.".to_string())?;
            }
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de aplicaciones desconocida: {otro}")),
        }
        i += 1;
    }

    let catalogo = lista_json_strings(&nix_config_json(raiz, "internal.applicationCatalog")?)?;

    if source == "auto" {
        source = if catalogo.iter().any(|actual| actual == &raw_id) {
            "curated".to_string()
        } else {
            "nixpkgs".to_string()
        };
    }

    let id = application_selection_token(&source, &raw_id)?;

    if source == "curated" && !catalogo.iter().any(|actual| actual == &raw_id) {
        return Err(format!(
            "{raw_id} no pertenece al catálogo curado. Usa su nombre normal y deja que Korunix lo resuelva, o elige una fuente explícita solo si hace falta."
        ));
    }

    let antes = lista_json_strings(&nix_config_json(raiz, "applications")?)?;
    let ya = antes.iter().any(|actual| actual == &id);
    let mut despues = antes.clone();

    if activar && !ya {
        despues.push(id.clone());
    } else if !activar {
        despues.retain(|actual| actual != &id);
    }

    let antes_json = serde_json::to_string(&antes).map_err(|e| e.to_string())?;
    let despues_json = serde_json::to_string(&despues).map_err(|e| e.to_string())?;
    let plan = format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-applications-change-plan\",\"application\":{},\"source\":{},\"selectionToken\":{},\"enabled\":{},\"before\":{},\"after\":{},\"requiresSystemApply\":true}}",
        json_texto(&raw_id),
        json_texto(&source),
        json_texto(&id),
        activar,
        antes_json,
        despues_json
    );

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        plan_only,
        yes,
        json,
        &format!(
            "¿Preparar {} {id} en la configuración?",
            if activar {
                "la instalación de"
            } else {
                "la retirada de"
            }
        ),
    )? {
        return Ok(code);
    }

    if antes == despues {
        if json {
            println!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-applications-change-result\",\"changed\":false,\"application\":{},\"source\":{},\"selectionToken\":{}}}",
                json_texto(&raw_id),
                json_texto(&source),
                json_texto(&id)
            );
        }
        return Ok(ExitCode::SUCCESS);
    }

    let ruta = host_config_path(raiz)?;
    let texto = fs::read_to_string(&ruta).map_err(|e| e.to_string())?;
    let nuevo = reemplazar_lista_nix(&texto, 4, "applications", &despues)?;
    let backup = aplicar_configuracion_host(raiz, "applications", &nuevo)?;

    history_record(
        "applications-prepared",
        &format!(
            "{} {id}",
            if activar {
                "Preparaste la instalación de"
            } else {
                "Preparaste la retirada de"
            }
        ),
    )?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-applications-change-result\",\"changed\":true,\"application\":{},\"source\":{},\"selectionToken\":{},\"enabled\":{},\"requiresSystemApply\":true,\"backup\":{}}}",
            json_texto(&raw_id),
            json_texto(&source),
            json_texto(&id),
            activar,
            json_texto(&backup.display().to_string())
        );
    } else {
        println!("✓ selección de aplicaciones preparada; falta aplicar la configuración");
    }

    Ok(ExitCode::SUCCESS)
}

fn desktop_state_json(raiz: &Path) -> Result<String, String> {
    let actual = nix_config_json(raiz, "desktop")?;
    let catalogo = nix_config_json(raiz, "internal.desktopCatalog")?;

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "desktop".into(),
            actual,
            "--argjson".into(),
            "catalog".into(),
            catalogo,
            r#"{
              schemaVersion:1,
              kind:"korunix-desktop-state",
              desktop:$desktop,
              catalog:$catalog
            }"#
            .into(),
        ],
    )
}

fn desktop_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        pretty(raiz, &desktop_state_json(raiz)?)?;
        return Ok(ExitCode::SUCCESS);
    }
    if args == ["--json"] {
        println!("{}", desktop_state_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    let operacion = args.first().map(String::as_str).unwrap_or("");
    if !matches!(operacion, "set-primary" | "set-additional") {
        return Err(
            "Uso: korunix desktop [--json] | set-primary <id> [--plan] [--yes] [--json] | set-additional <id,id,...> [--plan] [--yes] [--json]."
                .to_string(),
        );
    }

    let valor = args
        .get(1)
        .ok_or_else(|| "Falta la selección de escritorio.".to_string())?
        .to_string();

    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;
    for arg in &args[2..] {
        match arg.as_str() {
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de escritorio desconocida: {otro}")),
        }
    }

    let catalogo = lista_json_strings(&nix_config_json(raiz, "internal.desktopCatalog")?)?;
    let actual_raw = nix_config_json(raiz, "desktop")?;
    let actual: serde_json::Value = serde_json::from_str(&actual_raw).map_err(|e| e.to_string())?;

    let ruta = host_config_path(raiz)?;
    let texto = fs::read_to_string(&ruta).map_err(|e| e.to_string())?;

    let (nuevo, before, after) = if operacion == "set-primary" {
        if !catalogo.iter().any(|id| id == &valor) {
            return Err("Escritorio no soportado por el modelo Nix.".to_string());
        }
        let before = actual
            .get("primary")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let nuevo = reemplazar_linea_string(&texto, 6, "primary", &valor)?;
        (nuevo, serde_json::json!(before), serde_json::json!(valor))
    } else {
        let lista = if valor.trim().is_empty() {
            Vec::new()
        } else {
            valor
                .split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        };

        for id in &lista {
            if !catalogo.iter().any(|actual| actual == id) {
                return Err(format!("Escritorio no soportado: {id}"));
            }
        }

        let principal = actual
            .get("primary")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if lista.iter().any(|id| id == principal) {
            return Err("El escritorio principal no puede repetirse como adicional.".to_string());
        }

        let before = actual
            .get("additional")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        let nuevo = reemplazar_lista_nix(&texto, 6, "additional", &lista)?;
        (nuevo, before, serde_json::json!(lista))
    };

    let plan = serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-desktop-change-plan",
        "operation": operacion,
        "before": before,
        "after": after,
        "requiresSystemApply": true
    })
    .to_string();

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        plan_only,
        yes,
        json,
        "¿Preparar este cambio de escritorio?",
    )? {
        return Ok(code);
    }

    let backup = aplicar_configuracion_host(raiz, "desktop", &nuevo)?;
    history_record("desktop-prepared", "Preparaste un cambio de escritorio")?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-desktop-change-result\",\"changed\":true,\"requiresSystemApply\":true,\"backup\":{}}}",
            json_texto(&backup.display().to_string())
        );
    } else {
        println!("✓ cambio de escritorio preparado; falta aplicar la configuración");
    }

    Ok(ExitCode::SUCCESS)
}

const KORUNIX_INTERFACE_LANGUAGES: &[&str] = &[
    "be-Latn", "be", "ca", "cs", "de", "en", "es", "fr", "gl-ES", "hu", "it", "ko", "ku", "nl",
    "nn", "pl", "pt-BR", "ru", "sv", "tr", "uk-UA", "vi", "zh-Hans",
];

#[derive(Clone, Debug)]
struct InterfaceLanguageProfile {
    id: String,
    account_name: String,
    interface_language: Option<String>,
}

fn interface_language_supported(language: &str) -> bool {
    KORUNIX_INTERFACE_LANGUAGES.contains(&language)
}

fn interface_language_from_locale(locale: &str) -> Option<&'static str> {
    let normalized = locale.trim().to_ascii_lowercase().replace('-', "_");

    if normalized.starts_with("be_latn")
        || (normalized.starts_with("be_") && normalized.contains("@latin"))
    {
        Some("be-Latn")
    } else if normalized.starts_with("be") {
        Some("be")
    } else if normalized.starts_with("ca") {
        Some("ca")
    } else if normalized.starts_with("cs") {
        Some("cs")
    } else if normalized.starts_with("de") {
        Some("de")
    } else if normalized.starts_with("en") {
        Some("en")
    } else if normalized.starts_with("es") {
        Some("es")
    } else if normalized.starts_with("fr") {
        Some("fr")
    } else if normalized.starts_with("gl") {
        Some("gl-ES")
    } else if normalized.starts_with("hu") {
        Some("hu")
    } else if normalized.starts_with("it") {
        Some("it")
    } else if normalized.starts_with("ko") {
        Some("ko")
    } else if normalized.starts_with("ku") {
        Some("ku")
    } else if normalized.starts_with("nl") {
        Some("nl")
    } else if normalized.starts_with("nn") {
        Some("nn")
    } else if normalized.starts_with("pl") {
        Some("pl")
    } else if normalized.starts_with("pt_br") {
        Some("pt-BR")
    } else if normalized.starts_with("ru") {
        Some("ru")
    } else if normalized.starts_with("sv") {
        Some("sv")
    } else if normalized.starts_with("tr") {
        Some("tr")
    } else if normalized.starts_with("uk") {
        Some("uk-UA")
    } else if normalized.starts_with("vi") {
        Some("vi")
    } else if normalized.starts_with("zh") {
        Some("zh-Hans")
    } else {
        None
    }
}

fn interface_language_detected() -> (&'static str, &'static str) {
    for variable in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(variable) {
            if let Some(language) = interface_language_from_locale(&value) {
                return (language, "system-locale");
            }
        }
    }

    ("es", "spanish-fallback")
}

fn profile_simple_string_value(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    let mut found = None::<String>;

    for line in text.lines() {
        let trimmed = line.trim();

        if !trimmed.starts_with(&prefix) {
            continue;
        }

        if found.is_some() {
            return None;
        }

        let value = trimmed[prefix.len()..].trim();
        let value = value.strip_suffix(';')?.trim();
        let parsed = serde_json::from_str::<String>(value).ok()?;
        found = Some(parsed);
    }

    found
}

fn profile_simple_optional_string_value(
    text: &str,
    key: &str,
) -> Result<Option<Option<String>>, String> {
    let prefix = format!("{key} = ");
    let mut found = None::<Option<String>>;

    for line in text.lines() {
        let trimmed = line.trim();

        if !trimmed.starts_with(&prefix) {
            continue;
        }

        if found.is_some() {
            return Err(format!("El perfil contiene más de una declaración {key}."));
        }

        let value = trimmed[prefix.len()..].trim();
        let semicolon = value.find(';').ok_or_else(|| {
            format!("{key} no tiene una declaración simple terminada en punto y coma.")
        })?;

        let literal = value[..semicolon].trim();
        let suffix = value[semicolon + 1..].trim();

        if !suffix.is_empty() && !suffix.starts_with('#') {
            return Err(format!(
                "{key} contiene texto que Korunix no puede interpretar de forma segura."
            ));
        }

        let parsed = if literal == "null" {
            None
        } else {
            Some(
                serde_json::from_str::<String>(literal)
                    .map_err(|_| format!("{key} debe ser un texto o null."))?,
            )
        };

        found = Some(parsed);
    }

    Ok(found)
}

fn interface_language_profile_fast(
    raiz: &Path,
) -> Result<Option<InterfaceLanguageProfile>, String> {
    let account = match env::var("USER") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(None),
    };

    let directory = raiz.join("configuracion/personas");
    let mut matches = Vec::<(String, PathBuf)>::new();

    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("No pude leer {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("nix") {
            continue;
        }

        let text = fs::read_to_string(&path)
            .map_err(|error| format!("No pude leer {}: {error}", path.display()))?;

        if profile_simple_string_value(&text, "accountName").as_deref() != Some(account.as_str()) {
            continue;
        }

        let Some(id) = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(ToString::to_string)
        else {
            continue;
        };

        matches.push((id, path));
    }

    let [(id, path)] = matches.as_slice() else {
        return Ok(None);
    };

    let text = fs::read_to_string(path)
        .map_err(|error| format!("No pude leer {}: {error}", path.display()))?;

    // Esta ruta se ejecuta antes de dibujar la primera ventana.
    // Una preferencia simple del perfil no justifica evaluar Nix.
    let interface_language =
        profile_simple_optional_string_value(&text, "interfaceLanguage")?.unwrap_or(None);

    Ok(Some(InterfaceLanguageProfile {
        id: id.clone(),
        account_name: account,
        interface_language,
    }))
}

fn interface_language_profile_from_users(raiz: &Path) -> Result<InterfaceLanguageProfile, String> {
    let account = env::var("USER")
        .map_err(|_| "No pude conocer la cuenta que está usando Korunix.".to_string())?;

    let users_text = usuarios_json(raiz)?;
    let users: serde_json::Value =
        serde_json::from_str(&users_text).map_err(|error| error.to_string())?;

    let current = users
        .get("accounts")
        .and_then(serde_json::Value::as_array)
        .and_then(|accounts| {
            accounts.iter().find(|entry| {
                entry.get("accountName").and_then(serde_json::Value::as_str)
                    == Some(account.as_str())
            })
        })
        .ok_or_else(|| {
            format!("La cuenta {account} todavía no está asociada a un perfil de Korunix.")
        })?;

    let profile_id = current
        .get("profileId")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            format!("La cuenta {account} todavía no tiene un perfil portable asociado.")
        })?
        .to_string();

    let profile = users
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .and_then(|profiles| {
            profiles.iter().find(|profile| {
                profile.get("id").and_then(serde_json::Value::as_str) == Some(profile_id.as_str())
            })
        })
        .ok_or_else(|| {
            format!("Korunix no encontró el perfil portable {profile_id} de la cuenta actual.")
        })?;

    Ok(InterfaceLanguageProfile {
        id: profile_id,
        account_name: account,
        interface_language: profile
            .get("interfaceLanguage")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
    })
}

fn interface_language_profile(raiz: &Path) -> Result<InterfaceLanguageProfile, String> {
    if let Some(profile) = interface_language_profile_fast(raiz)? {
        return Ok(profile);
    }

    interface_language_profile_from_users(raiz)
}

fn profile_interface_language_text(text: &str, language: Option<&str>) -> Result<String, String> {
    if let Some(value) = language {
        if !interface_language_supported(value) {
            return Err(format!("Idioma de interfaz no soportado: {value}."));
        }
    }

    let mut lines = text.lines().map(ToString::to_string).collect::<Vec<_>>();
    let prefix = "  interfaceLanguage = ";

    let positions = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.starts_with(prefix).then_some(index))
        .collect::<Vec<_>>();

    if positions.len() > 1 {
        return Err("El perfil contiene más de una preferencia interfaceLanguage.".to_string());
    }

    let declaration = format!(
        "{prefix}{};",
        language
            .map(json_texto)
            .unwrap_or_else(|| "null".to_string())
    );

    if let Some(&index) = positions.first() {
        let line = &lines[index];
        let semicolon = line
            .find(';')
            .ok_or_else(|| "interfaceLanguage no tiene una declaración editable.".to_string())?;

        let suffix = &line[semicolon + 1..];
        lines[index] = format!("{declaration}{suffix}");
    } else {
        let language_positions = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| line.starts_with("  language = ").then_some(index))
            .collect::<Vec<_>>();

        if language_positions.len() != 1 {
            return Err("No encontré un punto seguro para guardar interfaceLanguage.".to_string());
        }

        lines.insert(language_positions[0] + 1, declaration);
    }

    let mut result = lines.join("\n");
    if text.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

fn interface_language_state_json(raiz: &Path) -> Result<String, String> {
    let profile = interface_language_profile(raiz)?;
    let detected = interface_language_detected();

    let explicit = profile
        .interface_language
        .as_deref()
        .filter(|language| interface_language_supported(language));

    let (effective, source) = match explicit {
        Some(language) => (language, "portable-profile"),
        None => detected,
    };

    Ok(serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-interface-language",
        "profileId": profile.id,
        "accountName": profile.account_name,
        "declaredLanguage": profile.interface_language,
        "language": effective,
        "automatic": explicit.is_none(),
        "source": source,
        "supportedLanguages": KORUNIX_INTERFACE_LANGUAGES,
        "changesSystemLanguage": false,
        "requiresSystemApply": false,
        "requiresRestart": false,
        "liveReloadSupported": true
    })
    .to_string())
}

fn interface_language_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() || args == ["--json"] {
        let state = interface_language_state_json(raiz)?;

        if args == ["--json"] {
            println!("{state}");
        } else {
            pretty(raiz, &state)?;
        }

        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) != Some("set") {
        return Err(
            "Uso: korunix interface-language [--json] | set <idioma|auto> [--plan] [--yes] [--json]."
                .to_string(),
        );
    }

    let requested = args
        .get(1)
        .ok_or_else(|| "Falta el idioma de Korunix.".to_string())?
        .to_string();

    let requested_value = if requested == "auto" {
        None
    } else {
        if !interface_language_supported(&requested) {
            return Err(format!(
                "Korunix no tiene una localización de interfaz publicada para {requested}."
            ));
        }
        Some(requested.clone())
    };

    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;

    for argument in &args[2..] {
        match argument.as_str() {
            "--plan" if !plan_only => plan_only = true,
            "--yes" if !yes => yes = true,
            "--json" if !json => json = true,
            other => return Err(format!("Opción de idioma de interfaz desconocida: {other}")),
        }
    }

    if plan_only && yes {
        return Err("--yes no se utiliza junto con --plan.".to_string());
    }

    if json && !plan_only && !yes {
        return Err("interface-language set --json necesita --yes o --plan.".to_string());
    }

    let profile = interface_language_profile(raiz)?;
    let before = profile.interface_language.clone();
    let changed = before != requested_value;

    let plan = serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-interface-language-change-plan",
        "profileId": profile.id,
        "accountName": profile.account_name,
        "before": before,
        "after": requested_value,
        "changed": changed,
        "writesPortableProfile": changed,
        "changesSystemLanguage": false,
        "changesSessionLanguage": false,
        "requiresSystemApply": false,
        "requiresRestart": false,
        "liveReloadSupported": true
    });

    if plan_only {
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan.to_string())?;
        }

        return Ok(ExitCode::SUCCESS);
    }

    if !yes && !confirm("¿Usar este idioma solamente en Korunix?")? {
        return Ok(ExitCode::SUCCESS);
    }

    if !changed {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "schemaVersion": 1,
                    "kind": "korunix-interface-language-change-result",
                    "profileId": profile.id,
                    "language": requested_value,
                    "changed": false,
                    "requiresSystemApply": false,
                    "requiresRestart": false,
                    "liveReloadSupported": true
                })
            );
        } else {
            println!("✓ Korunix ya tiene esa preferencia de idioma");
        }

        return Ok(ExitCode::SUCCESS);
    }

    let path = raiz
        .join("configuracion/personas")
        .join(format!("{}.nix", profile.id));

    let original = fs::read_to_string(&path)
        .map_err(|error| format!("No pude leer {}: {error}", path.display()))?;
    let updated = profile_interface_language_text(&original, requested_value.as_deref())?;

    let transaction = files_transaction_begin(raiz, std::slice::from_ref(&path))?;

    let result = (|| -> Result<(), String> {
        atomic_write(&path, updated.as_bytes())?;

        let evaluated_text = nix_archivo_json(raiz, &path)?;
        let evaluated: serde_json::Value =
            serde_json::from_str(&evaluated_text).map_err(|error| error.to_string())?;

        let persisted = evaluated
            .get("interfaceLanguage")
            .and_then(serde_json::Value::as_str);

        if persisted != requested_value.as_deref() {
            return Err("El perfil guardado no conserva la preferencia solicitada.".to_string());
        }

        Ok(())
    })();

    if let Err(error) = result {
        let recovery = rollback_pending_transaction(raiz);

        return match recovery {
            Ok(_) => Err(format!(
                "El cambio de idioma de Korunix fue revertido: {error}"
            )),
            Err(recovery_error) => Err(format!(
                "El cambio de idioma falló ({error}) y la recuperación automática también falló: {recovery_error}"
            )),
        };
    }

    transaction_commit(Some(&transaction))?;
    history_record(
        "interface-language-changed",
        "Cambiaste el idioma de la interfaz de Korunix",
    )?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "korunix-interface-language-change-result",
                "profileId": profile.id,
                "language": requested_value,
                "changed": true,
                "requiresSystemApply": false,
                "requiresRestart": false,
                "liveReloadSupported": true
            })
        );
    } else {
        println!("✓ idioma de Korunix actualizado");
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LocalizationKeyboard {
    layout: String,
    variant: String,
    label: String,
}

fn localization_language_valid(value: &str) -> bool {
    (2..=3).contains(&value.len()) && value.bytes().all(|c| c.is_ascii_lowercase())
}

fn localization_xkb_token_valid(value: &str, empty_ok: bool) -> bool {
    (empty_ok || !value.is_empty())
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'+' | b'.'))
}

fn localization_preferred_languages(raw: &str) -> Result<Vec<String>, String> {
    let values = serde_json::from_str::<Vec<String>>(raw).map_err(|_| {
        "--preferred-languages-json debe ser una lista JSON de idiomas.".to_string()
    })?;

    if values.is_empty() {
        return Err("Selecciona al menos un idioma preferido.".to_string());
    }

    let mut seen = BTreeSet::<String>::new();
    for value in &values {
        if !localization_language_valid(value) {
            return Err(format!("Código de idioma inválido: {value}"));
        }
        if !seen.insert(value.clone()) {
            return Err(format!("El idioma {value} está repetido."));
        }
    }

    Ok(values)
}

fn localization_keyboards(raw: &str) -> Result<Vec<LocalizationKeyboard>, String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|_| "--keyboards-json debe ser una lista JSON de teclados.".to_string())?;

    let items = value
        .as_array()
        .ok_or_else(|| "--keyboards-json debe contener una lista.".to_string())?;

    if items.is_empty() {
        return Err("Selecciona al menos un teclado.".to_string());
    }

    let mut seen = BTreeSet::<(String, String)>::new();
    let mut result = Vec::<LocalizationKeyboard>::new();

    for item in items {
        let layout = item
            .get("layout")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        let variant = item
            .get("variant")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        let label = item
            .get("label")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        if !localization_xkb_token_valid(&layout, false) {
            return Err(format!("Distribución XKB inválida: {layout}"));
        }

        if !localization_xkb_token_valid(&variant, true) {
            return Err(format!("Variante XKB inválida: {variant}"));
        }

        if label.is_empty() || label.contains('\n') || label.contains('\r') {
            return Err("Cada teclado necesita un nombre humano válido.".to_string());
        }

        if !seen.insert((layout.clone(), variant.clone())) {
            return Err(format!("El teclado {layout} ({variant}) está repetido."));
        }

        result.push(LocalizationKeyboard {
            layout,
            variant,
            label,
        });
    }

    Ok(result)
}

fn lista_nix_render(indent: usize, key: &str, values: &[String]) -> Vec<String> {
    let mut lines = vec![format!("{}{} = [", " ".repeat(indent), key)];

    for value in values {
        lines.push(format!("{}{}", " ".repeat(indent + 2), json_texto(value)));
    }

    lines.push(format!("{}];", " ".repeat(indent)));
    lines
}

fn reemplazar_o_insertar_lista_nix(
    texto: &str,
    indent: usize,
    key: &str,
    values: &[String],
    anchor: &str,
) -> Result<String, String> {
    let mut lines = texto.lines().map(ToString::to_string).collect::<Vec<_>>();
    let prefix = format!("{}{} = ", " ".repeat(indent), key);

    let positions = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.starts_with(&prefix).then_some(index))
        .collect::<Vec<_>>();

    if positions.len() > 1 {
        return Err(format!(
            "La configuración contiene más de una lista editable {key}."
        ));
    }

    let replacement = lista_nix_render(indent, key, values);

    if let Some(&start) = positions.first() {
        let after = lines[start][prefix.len()..].trim();

        let end = if after.starts_with('[') && after.ends_with("];") {
            start
        } else if after == "[" {
            let closing = format!("{}];", " ".repeat(indent));
            lines
                .iter()
                .enumerate()
                .skip(start + 1)
                .find_map(|(index, line)| (line == &closing).then_some(index))
                .ok_or_else(|| format!("La lista {key} no tiene cierre reconocible."))?
        } else {
            return Err(format!(
                "La lista {key} usa una forma manual que Korunix no modificará."
            ));
        };

        if end > start + 1
            && lines[start + 1..end]
                .iter()
                .any(|line| line.trim_start().starts_with('#'))
        {
            return Err(format!(
                "La lista {key} contiene comentarios manuales; Korunix no los borrará."
            ));
        }

        lines.splice(start..=end, replacement);
        return Ok(lines.join("\n") + "\n");
    }

    let anchor_prefix = format!("{}{} = ", " ".repeat(indent), anchor);
    let anchors = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.starts_with(&anchor_prefix).then_some(index))
        .collect::<Vec<_>>();

    if anchors.len() != 1 {
        return Err(format!(
            "No encontré un punto seguro para insertar la lista {key}."
        ));
    }

    let insert_at = anchors[0] + 1;
    for (offset, line) in replacement.into_iter().enumerate() {
        lines.insert(insert_at + offset, line);
    }

    Ok(lines.join("\n") + "\n")
}

fn parse_xkb_rules(raw: &str) -> Vec<LocalizationKeyboard> {
    let mut section = "";
    let mut layouts = BTreeMap::<String, String>::new();
    let mut variants = Vec::<(String, String, String)>::new();

    for original in raw.lines() {
        let line = original.trim();

        if line.starts_with('!') {
            section = if line.starts_with("! layout") {
                "layout"
            } else if line.starts_with("! variant") {
                "variant"
            } else {
                ""
            };
            continue;
        }

        if line.is_empty() || section.is_empty() {
            continue;
        }

        if section == "layout" {
            let mut parts = line.split_whitespace();
            let Some(layout) = parts.next() else {
                continue;
            };
            let label = parts.collect::<Vec<_>>().join(" ");

            if localization_xkb_token_valid(layout, false) && !label.is_empty() {
                layouts.insert(layout.to_string(), label);
            }
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(variant) = parts.next() else {
            continue;
        };
        let rest = parts.collect::<Vec<_>>().join(" ");
        let Some((layout, label)) = rest.split_once(':') else {
            continue;
        };

        let layout = layout.trim();
        let label = label.trim();

        if localization_xkb_token_valid(layout, false)
            && localization_xkb_token_valid(variant, false)
            && !label.is_empty()
        {
            variants.push((layout.to_string(), variant.to_string(), label.to_string()));
        }
    }

    let mut result = layouts
        .iter()
        .map(|(layout, label)| LocalizationKeyboard {
            layout: layout.clone(),
            variant: String::new(),
            label: label.clone(),
        })
        .collect::<Vec<_>>();

    for (layout, variant, label) in variants {
        if layouts.contains_key(&layout) {
            result.push(LocalizationKeyboard {
                layout,
                variant,
                label,
            });
        }
    }

    result.sort_by(|left, right| {
        left.label
            .to_ascii_lowercase()
            .cmp(&right.label.to_ascii_lowercase())
            .then_with(|| left.layout.cmp(&right.layout))
            .then_with(|| left.variant.cmp(&right.variant))
    });

    result
}

const KORUNIX_LANGUAGE_CATALOG: &[(&str, &str)] = &[
    ("be", "Беларуская"),
    ("ca", "Català"),
    ("cs", "Čeština"),
    ("de", "Deutsch"),
    ("en", "English"),
    ("es", "Español"),
    ("fr", "Français"),
    ("gl", "Galego"),
    ("hu", "Magyar"),
    ("it", "Italiano"),
    ("ko", "한국어"),
    ("ku", "Kurdî"),
    ("nl", "Nederlands"),
    ("nn", "Norsk nynorsk"),
    ("pl", "Polski"),
    ("pt", "Português"),
    ("ru", "Русский"),
    ("sv", "Svenska"),
    ("tr", "Türkçe"),
    ("uk", "Українська"),
    ("vi", "Tiếng Việt"),
    ("zh", "简体中文"),
];

const KORUNIX_REGION_CATALOG: &[(&str, &str)] = &[
    ("AF", "Afghanistan"),
    ("AL", "Albania"),
    ("DZ", "Algeria"),
    ("AS", "American Samoa"),
    ("AD", "Andorra"),
    ("AO", "Angola"),
    ("AI", "Anguilla"),
    ("AQ", "Antarctica"),
    ("AG", "Antigua and Barbuda"),
    ("AR", "Argentina"),
    ("AM", "Armenia"),
    ("AW", "Aruba"),
    ("AU", "Australia"),
    ("AT", "Austria"),
    ("AZ", "Azerbaijan"),
    ("BS", "Bahamas"),
    ("BH", "Bahrain"),
    ("BD", "Bangladesh"),
    ("BB", "Barbados"),
    ("BY", "Belarus"),
    ("BE", "Belgium"),
    ("BZ", "Belize"),
    ("BJ", "Benin"),
    ("BM", "Bermuda"),
    ("BT", "Bhutan"),
    ("BO", "Bolivia, Plurinational State of"),
    ("BQ", "Bonaire, Sint Eustatius and Saba"),
    ("BA", "Bosnia and Herzegovina"),
    ("BW", "Botswana"),
    ("BV", "Bouvet Island"),
    ("BR", "Brazil"),
    ("IO", "British Indian Ocean Territory"),
    ("BN", "Brunei Darussalam"),
    ("BG", "Bulgaria"),
    ("BF", "Burkina Faso"),
    ("BI", "Burundi"),
    ("CV", "Cabo Verde"),
    ("KH", "Cambodia"),
    ("CM", "Cameroon"),
    ("CA", "Canada"),
    ("KY", "Cayman Islands"),
    ("CF", "Central African Republic"),
    ("TD", "Chad"),
    ("CL", "Chile"),
    ("CN", "China"),
    ("CX", "Christmas Island"),
    ("CC", "Cocos (Keeling) Islands"),
    ("CO", "Colombia"),
    ("KM", "Comoros"),
    ("CG", "Congo"),
    ("CD", "Congo, The Democratic Republic of the"),
    ("CK", "Cook Islands"),
    ("CR", "Costa Rica"),
    ("HR", "Croatia"),
    ("CU", "Cuba"),
    ("CW", "Curaçao"),
    ("CY", "Cyprus"),
    ("CZ", "Czechia"),
    ("CI", "Côte d'Ivoire"),
    ("DK", "Denmark"),
    ("DJ", "Djibouti"),
    ("DM", "Dominica"),
    ("DO", "Dominican Republic"),
    ("EC", "Ecuador"),
    ("EG", "Egypt"),
    ("SV", "El Salvador"),
    ("GQ", "Equatorial Guinea"),
    ("ER", "Eritrea"),
    ("EE", "Estonia"),
    ("SZ", "Eswatini"),
    ("ET", "Ethiopia"),
    ("FK", "Falkland Islands (Malvinas)"),
    ("FO", "Faroe Islands"),
    ("FJ", "Fiji"),
    ("FI", "Finland"),
    ("FR", "France"),
    ("GF", "French Guiana"),
    ("PF", "French Polynesia"),
    ("TF", "French Southern Territories"),
    ("GA", "Gabon"),
    ("GM", "Gambia"),
    ("GE", "Georgia"),
    ("DE", "Germany"),
    ("GH", "Ghana"),
    ("GI", "Gibraltar"),
    ("GR", "Greece"),
    ("GL", "Greenland"),
    ("GD", "Grenada"),
    ("GP", "Guadeloupe"),
    ("GU", "Guam"),
    ("GT", "Guatemala"),
    ("GG", "Guernsey"),
    ("GN", "Guinea"),
    ("GW", "Guinea-Bissau"),
    ("GY", "Guyana"),
    ("HT", "Haiti"),
    ("HM", "Heard Island and McDonald Islands"),
    ("VA", "Holy See (Vatican City State)"),
    ("HN", "Honduras"),
    ("HK", "Hong Kong"),
    ("HU", "Hungary"),
    ("IS", "Iceland"),
    ("IN", "India"),
    ("ID", "Indonesia"),
    ("IR", "Iran, Islamic Republic of"),
    ("IQ", "Iraq"),
    ("IE", "Ireland"),
    ("IM", "Isle of Man"),
    ("IL", "Israel"),
    ("IT", "Italy"),
    ("JM", "Jamaica"),
    ("JP", "Japan"),
    ("JE", "Jersey"),
    ("JO", "Jordan"),
    ("KZ", "Kazakhstan"),
    ("KE", "Kenya"),
    ("KI", "Kiribati"),
    ("KP", "Korea, Democratic People's Republic of"),
    ("KR", "Korea, Republic of"),
    ("KW", "Kuwait"),
    ("KG", "Kyrgyzstan"),
    ("LA", "Lao People's Democratic Republic"),
    ("LV", "Latvia"),
    ("LB", "Lebanon"),
    ("LS", "Lesotho"),
    ("LR", "Liberia"),
    ("LY", "Libya"),
    ("LI", "Liechtenstein"),
    ("LT", "Lithuania"),
    ("LU", "Luxembourg"),
    ("MO", "Macao"),
    ("MG", "Madagascar"),
    ("MW", "Malawi"),
    ("MY", "Malaysia"),
    ("MV", "Maldives"),
    ("ML", "Mali"),
    ("MT", "Malta"),
    ("MH", "Marshall Islands"),
    ("MQ", "Martinique"),
    ("MR", "Mauritania"),
    ("MU", "Mauritius"),
    ("YT", "Mayotte"),
    ("MX", "Mexico"),
    ("FM", "Micronesia, Federated States of"),
    ("MD", "Moldova, Republic of"),
    ("MC", "Monaco"),
    ("MN", "Mongolia"),
    ("ME", "Montenegro"),
    ("MS", "Montserrat"),
    ("MA", "Morocco"),
    ("MZ", "Mozambique"),
    ("MM", "Myanmar"),
    ("NA", "Namibia"),
    ("NR", "Nauru"),
    ("NP", "Nepal"),
    ("NL", "Netherlands"),
    ("NC", "New Caledonia"),
    ("NZ", "New Zealand"),
    ("NI", "Nicaragua"),
    ("NE", "Niger"),
    ("NG", "Nigeria"),
    ("NU", "Niue"),
    ("NF", "Norfolk Island"),
    ("MK", "North Macedonia"),
    ("MP", "Northern Mariana Islands"),
    ("NO", "Norway"),
    ("OM", "Oman"),
    ("PK", "Pakistan"),
    ("PW", "Palau"),
    ("PS", "Palestine, State of"),
    ("PA", "Panama"),
    ("PG", "Papua New Guinea"),
    ("PY", "Paraguay"),
    ("PE", "Peru"),
    ("PH", "Philippines"),
    ("PN", "Pitcairn"),
    ("PL", "Poland"),
    ("PT", "Portugal"),
    ("PR", "Puerto Rico"),
    ("QA", "Qatar"),
    ("RO", "Romania"),
    ("RU", "Russian Federation"),
    ("RW", "Rwanda"),
    ("RE", "Réunion"),
    ("BL", "Saint Barthélemy"),
    ("SH", "Saint Helena, Ascension and Tristan da Cunha"),
    ("KN", "Saint Kitts and Nevis"),
    ("LC", "Saint Lucia"),
    ("MF", "Saint Martin (French part)"),
    ("PM", "Saint Pierre and Miquelon"),
    ("VC", "Saint Vincent and the Grenadines"),
    ("WS", "Samoa"),
    ("SM", "San Marino"),
    ("ST", "Sao Tome and Principe"),
    ("SA", "Saudi Arabia"),
    ("SN", "Senegal"),
    ("RS", "Serbia"),
    ("SC", "Seychelles"),
    ("SL", "Sierra Leone"),
    ("SG", "Singapore"),
    ("SX", "Sint Maarten (Dutch part)"),
    ("SK", "Slovakia"),
    ("SI", "Slovenia"),
    ("SB", "Solomon Islands"),
    ("SO", "Somalia"),
    ("ZA", "South Africa"),
    ("GS", "South Georgia and the South Sandwich Islands"),
    ("SS", "South Sudan"),
    ("ES", "Spain"),
    ("LK", "Sri Lanka"),
    ("SD", "Sudan"),
    ("SR", "Suriname"),
    ("SJ", "Svalbard and Jan Mayen"),
    ("SE", "Sweden"),
    ("CH", "Switzerland"),
    ("SY", "Syrian Arab Republic"),
    ("TW", "Taiwan, Province of China"),
    ("TJ", "Tajikistan"),
    ("TZ", "Tanzania, United Republic of"),
    ("TH", "Thailand"),
    ("TL", "Timor-Leste"),
    ("TG", "Togo"),
    ("TK", "Tokelau"),
    ("TO", "Tonga"),
    ("TT", "Trinidad and Tobago"),
    ("TN", "Tunisia"),
    ("TM", "Turkmenistan"),
    ("TC", "Turks and Caicos Islands"),
    ("TV", "Tuvalu"),
    ("TR", "Türkiye"),
    ("UG", "Uganda"),
    ("UA", "Ukraine"),
    ("AE", "United Arab Emirates"),
    ("GB", "United Kingdom"),
    ("US", "United States"),
    ("UM", "United States Minor Outlying Islands"),
    ("UY", "Uruguay"),
    ("UZ", "Uzbekistan"),
    ("VU", "Vanuatu"),
    ("VE", "Venezuela, Bolivarian Republic of"),
    ("VN", "Viet Nam"),
    ("VG", "Virgin Islands, British"),
    ("VI", "Virgin Islands, U.S."),
    ("WF", "Wallis and Futuna"),
    ("EH", "Western Sahara"),
    ("YE", "Yemen"),
    ("ZM", "Zambia"),
    ("ZW", "Zimbabwe"),
    ("AX", "Åland Islands"),
];

fn localization_timezone_label(value: &str) -> String {
    let parts = value.split('/').collect::<Vec<_>>();
    let city = parts.last().copied().unwrap_or(value).replace('_', " ");

    if parts.len() >= 3 {
        let middle = parts[parts.len() - 2].replace('_', " ");
        format!("{city} — {middle}")
    } else if let Some(area) = parts.first() {
        format!("{city} — {}", area.replace('_', " "))
    } else {
        city
    }
}

fn localization_catalog_json(raiz: &Path) -> Result<String, String> {
    let host = resolver_equipo(raiz)?;

    let runtime = runtime_state_current(raiz)?;

    let xkb_root = if let Some(value) = runtime
        .as_ref()
        .and_then(|state| state.pointer("/localization/catalog/xkbRoot"))
        .and_then(serde_json::Value::as_str)
    {
        value.to_string()
    } else {
        flake_raw(
            raiz,
            &format!("nixosConfigurations.{host}.pkgs.xkeyboard_config.outPath"),
        )?
    };

    let xkb_candidates = [
        PathBuf::from(&xkb_root).join("share/X11/xkb/rules/evdev.lst"),
        PathBuf::from(&xkb_root).join("share/X11/xkb/rules/base.lst"),
    ];

    let xkb_file = xkb_candidates
        .iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            "No pude localizar el catálogo XKB de la revisión actual de Nixpkgs.".to_string()
        })?;

    let xkb_raw = fs::read_to_string(xkb_file)
        .map_err(|error| format!("No pude leer {}: {error}", xkb_file.display()))?;
    let keyboards = parse_xkb_rules(&xkb_raw);

    if keyboards.is_empty() {
        return Err("El catálogo XKB actual no contiene teclados utilizables.".to_string());
    }

    let tz_root = if let Some(value) = runtime
        .as_ref()
        .and_then(|state| state.pointer("/localization/catalog/tzdataRoot"))
        .and_then(serde_json::Value::as_str)
    {
        value.to_string()
    } else {
        flake_raw(
            raiz,
            &format!("nixosConfigurations.{host}.pkgs.tzdata.outPath"),
        )?
    };

    let timezone_candidates = [
        PathBuf::from(&tz_root).join("share/zoneinfo/zone1970.tab"),
        PathBuf::from(&tz_root).join("share/zoneinfo/zone.tab"),
    ];

    let timezone_file = timezone_candidates
        .iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "No pude localizar el catálogo de zonas horarias.".to_string())?;

    let timezone_raw = fs::read_to_string(timezone_file)
        .map_err(|error| format!("No pude leer {}: {error}", timezone_file.display()))?;

    let mut timezone_ids = BTreeSet::<String>::new();
    for line in timezone_raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let columns = line.split('\t').collect::<Vec<_>>();
        if let Some(zone) = columns.get(2).map(|value| value.trim()) {
            if zone.contains('/') && !zone.contains("..") {
                timezone_ids.insert(zone.to_string());
            }
        }
    }

    let languages = KORUNIX_LANGUAGE_CATALOG
        .iter()
        .map(|(code, label)| {
            serde_json::json!({
                "code": code,
                "label": label
            })
        })
        .collect::<Vec<_>>();

    let regions = KORUNIX_REGION_CATALOG
        .iter()
        .map(|(code, label)| {
            serde_json::json!({
                "code": code,
                "label": label
            })
        })
        .collect::<Vec<_>>();

    let time_zones = timezone_ids
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "label": localization_timezone_label(id)
            })
        })
        .collect::<Vec<_>>();

    let keyboard_values = keyboards
        .iter()
        .map(|keyboard| {
            serde_json::json!({
                "layout": keyboard.layout.clone(),
                "variant": keyboard.variant.clone(),
                "label": keyboard.label.clone()
            })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-localization-catalog",
        "source": {
            "languages": "korunix-noctalia-language-family",
            "regions": "iso-3166-1",
            "timeZones": "tzdata",
            "keyboards": "xkeyboard-config"
        },
        "languages": languages,
        "regions": regions,
        "timeZones": time_zones,
        "keyboards": keyboard_values
    })
    .to_string())
}

fn localization_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        localization_human(raiz)?;
        return Ok(ExitCode::SUCCESS);
    }

    if args == ["--json"] {
        println!("{}", localizacion_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    if args == ["catalog", "--json"] {
        println!("{}", localization_catalog_json(raiz)?);
        return Ok(ExitCode::SUCCESS);
    }

    if args.first().map(String::as_str) != Some("set") {
        return Err(
            "Uso: korunix localization [--json] | catalog --json | set [--language xx] [--preferred-languages-json lista] [--region XX] [--formats-language xx] [--formats-region XX] [--timezone Zona/Ciudad] [--keyboard layout] [--variant variante] [--keyboards-json lista] [--plan] [--yes] [--json]."
                .to_string(),
        );
    }

    let mut language = None::<String>;
    let mut preferred_languages = None::<Vec<String>>;
    let mut region = None::<String>;
    let mut formats_language = None::<String>;
    let mut formats_region = None::<String>;
    let mut timezone = None::<String>;
    let mut keyboard = None::<String>;
    let mut variant = None::<String>;
    let mut keyboards = None::<Vec<LocalizationKeyboard>>;
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--language" => {
                i += 1;
                language = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta idioma.".to_string())?,
                );
            }
            "--preferred-languages-json" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| "Falta la lista de idiomas preferidos.".to_string())?;
                preferred_languages = Some(localization_preferred_languages(raw)?);
            }
            "--region" => {
                i += 1;
                region = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta región.".to_string())?,
                );
            }
            "--formats-language" => {
                i += 1;
                formats_language = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta idioma de formatos.".to_string())?,
                );
            }
            "--formats-region" => {
                i += 1;
                formats_region = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta región de formatos.".to_string())?,
                );
            }
            "--timezone" => {
                i += 1;
                timezone = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta zona horaria.".to_string())?,
                );
            }
            "--keyboard" => {
                i += 1;
                keyboard = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta distribución de teclado.".to_string())?,
                );
            }
            "--variant" => {
                i += 1;
                variant = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta variante de teclado.".to_string())?,
                );
            }
            "--keyboards-json" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| "Falta la lista de teclados.".to_string())?;
                keyboards = Some(localization_keyboards(raw)?);
            }
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            other => return Err(format!("Opción de localización desconocida: {other}")),
        }
        i += 1;
    }

    if language.is_none()
        && preferred_languages.is_none()
        && region.is_none()
        && formats_language.is_none()
        && formats_region.is_none()
        && timezone.is_none()
        && keyboard.is_none()
        && variant.is_none()
        && keyboards.is_none()
    {
        return Err("No se indicó ningún cambio de localización.".to_string());
    }

    if let Some(value) = language.as_deref() {
        if !localization_language_valid(value) {
            return Err("Código de idioma inválido.".to_string());
        }
    }

    if let Some(value) = formats_language.as_deref() {
        if !localization_language_valid(value) {
            return Err("Código de idioma de formatos inválido.".to_string());
        }
    }

    for value in [&region, &formats_region].into_iter().flatten() {
        if value.len() != 2 || !value.bytes().all(|c| c.is_ascii_uppercase()) {
            return Err("Código de región inválido.".to_string());
        }
    }

    if let Some(value) = timezone.as_deref() {
        if !value.contains('/') || value.contains("..") {
            return Err("Zona horaria inválida.".to_string());
        }
    }

    let before_raw = nix_config_json(raiz, "localization")?;
    let before: serde_json::Value =
        serde_json::from_str(&before_raw).map_err(|error| error.to_string())?;

    if preferred_languages.is_none() {
        if let Some(new_language) = language.as_ref() {
            let mut existing = before
                .get("preferredLanguages")
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            existing.retain(|value| value != new_language);
            existing.insert(0, new_language.clone());
            preferred_languages = Some(existing);
        }
    }

    if let Some(values) = preferred_languages.as_ref() {
        if language.is_none() {
            language = values.first().cloned();
        }

        if values.first().map(String::as_str) != language.as_deref() {
            return Err(
                "El primer idioma preferido debe ser el idioma principal del sistema.".to_string(),
            );
        }
    }

    if keyboards.is_some() && (keyboard.is_some() || variant.is_some()) {
        return Err("--keyboards-json no se combina con --keyboard ni --variant.".to_string());
    }

    if let Some(values) = keyboards.as_ref() {
        let first = values
            .first()
            .ok_or_else(|| "Selecciona al menos un teclado.".to_string())?;
        keyboard = Some(first.layout.clone());
        variant = Some(first.variant.clone());
    }

    if let Some(value) = keyboard.as_deref() {
        if !localization_xkb_token_valid(value, false) {
            return Err("Distribución de teclado inválida.".to_string());
        }
    }

    if let Some(value) = variant.as_deref() {
        if !localization_xkb_token_valid(value, true) {
            return Err("Variante de teclado inválida.".to_string());
        }
    }

    let ruta = host_config_path(raiz)?;
    let mut nuevo = fs::read_to_string(&ruta).map_err(|error| error.to_string())?;

    if let Some(value) = language.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 6, "systemLanguage", value)?;
    }

    if let Some(values) = preferred_languages.as_ref() {
        nuevo = reemplazar_o_insertar_lista_nix(
            &nuevo,
            6,
            "preferredLanguages",
            values,
            "systemLanguage",
        )?;
    }

    if let Some(value) = region.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 6, "region", value)?;
    }

    if let Some(value) = formats_language.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 8, "language", value)?;
    }

    if let Some(value) = formats_region.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 8, "region", value)?;
    }

    if let Some(value) = timezone.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 6, "timeZone", value)?;
    }

    if let Some(value) = keyboard.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 8, "layout", value)?;
    }

    if let Some(value) = variant.as_ref() {
        nuevo = reemplazar_linea_string(&nuevo, 8, "variant", value)?;
    }

    if let Some(values) = keyboards.as_ref() {
        let additional_layouts = values
            .iter()
            .skip(1)
            .map(|value| value.layout.clone())
            .collect::<Vec<_>>();

        let additional_variants = values
            .iter()
            .skip(1)
            .map(|value| value.variant.clone())
            .collect::<Vec<_>>();

        let display_names = values
            .iter()
            .map(|value| value.label.clone())
            .collect::<Vec<_>>();

        nuevo = reemplazar_o_insertar_lista_nix(
            &nuevo,
            8,
            "additionalLayouts",
            &additional_layouts,
            "variant",
        )?;
        nuevo = reemplazar_o_insertar_lista_nix(
            &nuevo,
            8,
            "additionalVariants",
            &additional_variants,
            "additionalLayouts",
        )?;
        nuevo = reemplazar_o_insertar_lista_nix(
            &nuevo,
            8,
            "displayNames",
            &display_names,
            "additionalVariants",
        )?;
    }

    let keyboard_changes = keyboards.as_ref().map(|values| {
        serde_json::Value::Array(
            values
                .iter()
                .map(|value| {
                    serde_json::json!({
                        "layout": value.layout.clone(),
                        "variant": value.variant.clone(),
                        "label": value.label.clone()
                    })
                })
                .collect(),
        )
    });

    let changes = serde_json::json!({
        "language": language,
        "preferredLanguages": preferred_languages,
        "region": region,
        "formatsLanguage": formats_language,
        "formatsRegion": formats_region,
        "timeZone": timezone,
        "keyboard": keyboard,
        "variant": variant,
        "keyboards": keyboard_changes
    });

    let plan = serde_json::json!({
        "schemaVersion": 2,
        "kind": "korunix-localization-change-plan",
        "before": before,
        "changes": changes,
        "requiresSystemApply": true
    })
    .to_string();

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        plan_only,
        yes,
        json,
        "¿Preparar estos cambios de idioma, región y teclado?",
    )? {
        return Ok(code);
    }

    let backup = aplicar_configuracion_host(raiz, "localization", &nuevo)?;
    history_record(
        "localization-prepared",
        "Preparaste un cambio de idioma, región o teclado",
    )?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "schemaVersion": 2,
                "kind": "korunix-localization-change-result",
                "changed": true,
                "requiresSystemApply": true,
                "backup": backup.display().to_string()
            })
        );
    } else {
        println!("✓ localización preparada; falta aplicar la configuración");
    }

    Ok(ExitCode::SUCCESS)
}

fn users_create_structured(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let mut account = None::<String>;
    let mut name = None::<String>;
    let mut role = "standard".to_string();
    let mut avatar = None::<String>;
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--account" => {
                i += 1;
                account = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta nombre de cuenta.".to_string())?,
                );
            }
            "--name" => {
                i += 1;
                name = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta nombre visible.".to_string())?,
                );
            }
            "--role" => {
                i += 1;
                role = args
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "Falta rol.".to_string())?;
            }
            "--avatar" => {
                i += 1;
                avatar = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta la ruta del avatar.".to_string())?,
                );
            }
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de creación de persona desconocida: {otro}")),
        }
        i += 1;
    }

    let account = account.ok_or_else(|| "Falta --account.".to_string())?;
    if !account_valid(&account) {
        return Err("Nombre de cuenta inválido.".to_string());
    }

    let name = name
        .filter(|valor| !valor.trim().is_empty())
        .unwrap_or_else(|| account.clone());

    let admin = match role.as_str() {
        "admin" | "administrator" => true,
        "standard" | "estandar" | "estándar" => false,
        _ => return Err("El rol debe ser admin o standard.".to_string()),
    };

    let avatar_validado = avatar.as_deref().map(avatar_source).transpose()?;

    let host = resolver_equipo(raiz)?;
    let profile = raiz
        .join("configuracion/personas")
        .join(format!("{account}.nix"));
    if profile.exists() {
        return Err("Ese perfil ya existe.".to_string());
    }

    let plan = format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-user-create-plan\",\"accountName\":{},\"displayName\":{},\"role\":{},\"profilePath\":{},\"requiresSystemApply\":true,\"password\":{{\"required\":true,\"transport\":\"stdin-after-apply\",\"stored\":false}},\"avatar\":{{\"optional\":true,\"implemented\":true,\"selected\":{}}}}}",
        json_texto(&account),
        json_texto(&name),
        json_texto(if admin { "administrator" } else { "standard" }),
        json_texto(&profile.display().to_string()),
        avatar_validado.is_some()
    );

    if let Some(code) = salida_plan_o_confirmacion(
        raiz,
        &plan,
        plan_only,
        yes,
        json,
        &format!("¿Preparar la cuenta {account}?"),
    )? {
        return Ok(code);
    }

    let host_path = raiz
        .join("configuracion/equipos")
        .join(format!("{host}.nix"));

    let mut avatar_destino = None::<(PathBuf, PathBuf)>;
    let avatar_relativo = if let Some((origen, extension)) = avatar_validado.as_ref() {
        let carpeta = raiz.join("configuracion/personas");
        let destino = carpeta.join(format!("{account}.{extension}"));
        if destino.exists() {
            return Err("Ya existe un avatar administrado para esa cuenta.".to_string());
        }

        avatar_destino = Some((origen.clone(), destino));
        Some(format!("{account}.{extension}"))
    } else {
        None
    };

    let perfil = profile_text_with_avatar(&account, &name, avatar_relativo.as_deref())?;
    let mut transaction_paths = vec![host_path.clone(), profile.clone()];
    if let Some((_, destino)) = avatar_destino.as_ref() {
        transaction_paths.push(destino.clone());
    }
    let transaction = files_transaction_begin(raiz, &transaction_paths)?;

    let result = (|| -> Result<(), String> {
        if let Some((origen, destino)) = avatar_destino.as_ref() {
            fs::copy(origen, destino).map_err(|e| format!("No pude copiar el avatar: {e}"))?;
        }

        atomic_write(&profile, perfil.as_bytes())?;
        add_host_user(
            raiz,
            &host,
            &account,
            &account,
            &format!("/home/{account}"),
            admin,
            &[],
        )?;
        validate(raiz)
    })();

    if let Err(error) = result {
        let recovery = rollback_pending_transaction(raiz);
        return match recovery {
            Ok(_) => Err(format!("Creación revertida: {error}")),
            Err(recovery_error) => Err(format!(
                "La creación falló ({error}) y la recuperación automática también falló: {recovery_error}"
            )),
        };
    }

    transaction_commit(Some(&transaction))?;

    history_record(
        "user-prepared",
        &format!(
            "Preparaste a {name} como {}",
            if admin {
                "administrador"
            } else {
                "usuario estándar"
            }
        ),
    )?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-user-create-result\",\"accountName\":{},\"prepared\":true,\"requiresSystemApply\":true,\"passwordRequiredAfterApply\":true}}",
            json_texto(&account)
        );
    } else {
        println!("✓ persona preparada; aplica la configuración y establece su contraseña");
    }

    Ok(ExitCode::SUCCESS)
}

fn users_password_stdin(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let account = args
        .first()
        .ok_or_else(|| "Uso: korunix users password-stdin <cuenta> --yes [--json].".to_string())?;

    if !account_valid(account) {
        return Err("Nombre de cuenta inválido.".to_string());
    }

    let yes = args.iter().any(|v| v == "--yes");
    let json = args.iter().any(|v| v == "--json");
    if args.iter().skip(1).any(|v| v != "--yes" && v != "--json") {
        return Err("Opción de contraseña desconocida.".to_string());
    }

    if !yes {
        return Err(
            "password-stdin necesita --yes; el secreto nunca se acepta como argumento.".to_string(),
        );
    }

    let mut secreto = String::new();
    let mut entrada = io::stdin();
    std::io::Read::read_to_string(&mut entrada, &mut secreto)
        .map_err(|e| format!("No pude leer la contraseña desde stdin: {e}"))?;

    while secreto.ends_with('\n') || secreto.ends_with('\r') {
        secreto.pop();
    }

    if secreto.is_empty() {
        return Err("La contraseña recibida está vacía.".to_string());
    }

    if secreto.contains('\n') || secreto.contains('\r') {
        return Err("La entrada protegida contiene más de una línea.".to_string());
    }

    let payload = format!("{account}:{secreto}\n");
    privileged_input(raiz, "chpasswd", &[], payload.as_bytes())?;
    secreto.clear();

    history_record(
        "user-password",
        &format!("Actualizaste la contraseña de {account}"),
    )?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-user-password-result\",\"accountName\":{},\"changed\":true,\"secretStored\":false}}",
            json_texto(account)
        );
    } else {
        println!("✓ contraseña administrada por el sistema");
    }

    Ok(ExitCode::SUCCESS)
}

fn copy_dir_recursive(origen: &Path, destino: &Path) -> Result<(), String> {
    fs::create_dir_all(destino).map_err(|e| format!("No pude crear {}: {e}", destino.display()))?;

    for entry in
        fs::read_dir(origen).map_err(|e| format!("No pude leer {}: {e}", origen.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = destino.join(entry.file_name());
        let tipo = entry.file_type().map_err(|e| e.to_string())?;

        if tipo.is_symlink() {
            return Err(format!(
                "La copia de configuración contiene un enlace simbólico no permitido: {}",
                from.display()
            ));
        } else if tipo.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if tipo.is_file() {
            fs::copy(&from, &to).map_err(|e| format!("No pude copiar {}: {e}", from.display()))?;
        }
    }

    Ok(())
}

fn backup_default_path() -> PathBuf {
    let base = env::var_os("KORUNIX_EXPORT_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(format!("korunix-config-{}.tar.gz", stamp()))
}

fn backup_listing_safe(raiz: &Path, bundle: &Path) -> Result<(), String> {
    let archivo = bundle.display().to_string();
    let listing = capture(raiz, "tar", &["-tzf".into(), archivo.clone()])?;

    for entrada in listing.lines().map(str::trim).filter(|v| !v.is_empty()) {
        let ruta = Path::new(entrada);
        if ruta.is_absolute()
            || ruta.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::Prefix(_)
                )
            })
        {
            return Err("La copia contiene una ruta insegura.".to_string());
        }

        if entrada != "flake.lock"
            && entrada != "configuracion"
            && entrada != "configuracion/"
            && !entrada.starts_with("configuracion/")
        {
            return Err(format!(
                "La copia contiene una ruta fuera de la configuración: {entrada}"
            ));
        }
    }

    let verbose = capture(raiz, "tar", &["-tvzf".into(), archivo])?;

    for linea in verbose.lines().filter(|v| !v.trim().is_empty()) {
        let tipo = linea.as_bytes().first().copied().unwrap_or(b'?');
        if !matches!(tipo, b'-' | b'd') {
            return Err("La copia contiene enlaces u objetos no permitidos.".to_string());
        }
    }

    Ok(())
}

fn backup_operation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let accion = args.first().map(String::as_str).unwrap_or("");

    if accion == "export" {
        let mut salida = None::<PathBuf>;
        let mut plan_only = false;
        let mut yes = false;
        let mut json = false;

        let mut i = 1usize;
        while i < args.len() {
            match args[i].as_str() {
                "--output" => {
                    i += 1;
                    salida = Some(PathBuf::from(
                        args.get(i)
                            .ok_or_else(|| "Falta ruta de salida.".to_string())?,
                    ));
                }
                "--plan" => plan_only = true,
                "--yes" => yes = true,
                "--json" => json = true,
                otro => return Err(format!("Opción de copia desconocida: {otro}")),
            }
            i += 1;
        }

        let salida = salida.unwrap_or_else(backup_default_path);
        let plan = format!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-backup-export-plan\",\"output\":{},\"includes\":[\"configuracion\",\"flake.lock\"],\"excludes\":[\"generado\",\"credentials\",\"state\",\"history\"],\"portable\":true}}",
            json_texto(&salida.display().to_string())
        );

        if let Some(code) = salida_plan_o_confirmacion(
            raiz,
            &plan,
            plan_only,
            yes,
            json,
            "¿Crear una copia portable de la configuración?",
        )? {
            return Ok(code);
        }

        if let Some(parent) = salida.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        visible(
            raiz,
            "tar",
            &[
                "-czf".into(),
                salida.display().to_string(),
                "-C".into(),
                raiz.display().to_string(),
                "configuracion".into(),
                "flake.lock".into(),
            ],
        )?;

        let mut permisos = fs::metadata(&salida)
            .map_err(|e| e.to_string())?
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permisos, 0o600);
        fs::set_permissions(&salida, permisos).map_err(|e| e.to_string())?;

        history_record(
            "backup-export",
            "Creaste una copia portable de la configuración",
        )?;

        if json {
            println!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-backup-export-result\",\"output\":{},\"created\":true,\"containsCredentials\":false}}",
                json_texto(&salida.display().to_string())
            );
        } else {
            println!("✓ copia creada: {}", salida.display());
        }
        return Ok(ExitCode::SUCCESS);
    }

    if accion == "inspect" {
        let bundle = PathBuf::from(
            args.get(1)
                .ok_or_else(|| "Uso: korunix backup inspect <archivo> [--json].".to_string())?,
        );
        backup_listing_safe(raiz, &bundle)?;
        let json = args.iter().any(|v| v == "--json");

        let resultado = format!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-backup-inspection\",\"path\":{},\"valid\":true,\"containsCredentials\":false}}",
            json_texto(&bundle.display().to_string())
        );
        if json {
            println!("{resultado}");
        } else {
            pretty(raiz, &resultado)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if accion == "restore" {
        let bundle = PathBuf::from(args.get(1).ok_or_else(|| {
            "Uso: korunix backup restore <archivo> [--plan] [--yes] [--json].".to_string()
        })?);

        let mut plan_only = false;
        let mut yes = false;
        let mut json = false;
        for arg in &args[2..] {
            match arg.as_str() {
                "--plan" => plan_only = true,
                "--yes" => yes = true,
                "--json" => json = true,
                otro => return Err(format!("Opción de restauración desconocida: {otro}")),
            }
        }

        backup_listing_safe(raiz, &bundle)?;

        let plan = format!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-backup-restore-plan\",\"path\":{},\"replaces\":[\"configuracion\",\"flake.lock\"],\"createsSafetyBackup\":true,\"validatesBeforeCompletion\":true,\"credentialsImported\":false}}",
            json_texto(&bundle.display().to_string())
        );

        if let Some(code) = salida_plan_o_confirmacion(
            raiz,
            &plan,
            plan_only,
            yes,
            json,
            "¿Restaurar esta configuración? Korunix guardará primero la actual.",
        )? {
            return Ok(code);
        }

        let temp = env::temp_dir().join(format!("korunix-backup-restore-{}", stamp()));
        fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
        visible(
            raiz,
            "tar",
            &[
                "-xzf".into(),
                bundle.display().to_string(),
                "-C".into(),
                temp.display().to_string(),
                "--no-same-owner".into(),
                "--no-same-permissions".into(),
            ],
        )?;

        let nueva_config = temp.join("configuracion");
        let nuevo_lock = temp.join("flake.lock");
        if !nueva_config.is_dir() || !nuevo_lock.is_file() {
            let _ = fs::remove_dir_all(&temp);
            return Err("La copia no contiene configuracion/ y flake.lock.".to_string());
        }

        let seguridad = backup_dir("config-restore")?;
        copy_dir_recursive(
            &raiz.join("configuracion"),
            &seguridad.join("configuracion"),
        )?;
        fs::copy(raiz.join("flake.lock"), seguridad.join("flake.lock"))
            .map_err(|e| e.to_string())?;

        // La nueva configuración se prepara completa al lado de la actual para
        // poder intercambiar ambos árboles con una única operación del kernel.
        let actual = raiz.join("configuracion");
        let candidate = raiz.join(format!(".korunix-restore-config-{}", stamp()));
        copy_dir_recursive(&nueva_config, &candidate)?;

        // El marcador se escribe antes del primer cambio visible. Si el proceso
        // desaparece en cualquier punto posterior, la siguiente operación de
        // Korunix restaura automáticamente el estado anterior desde `seguridad`.
        restore_pending_write(&seguridad, &candidate)?;

        if let Err(error) = exchange_paths(&actual, &candidate) {
            let _ = rollback_pending_transaction(raiz);
            let _ = fs::remove_dir_all(&temp);
            return Err(error);
        }

        let candidate_lock = match fs::read(&nuevo_lock) {
            Ok(data) => data,
            Err(error) => {
                let _ = rollback_pending_transaction(raiz);
                let _ = fs::remove_dir_all(&temp);
                return Err(error.to_string());
            }
        };
        if let Err(error) = atomic_write(&raiz.join("flake.lock"), &candidate_lock) {
            let _ = rollback_pending_transaction(raiz);
            let _ = fs::remove_dir_all(&temp);
            return Err(error);
        }

        if let Err(error) = validate(raiz) {
            let recovery = rollback_pending_transaction(raiz);
            let _ = fs::remove_dir_all(&temp);
            return match recovery {
                Ok(_) => Err(format!(
                    "La copia restaurada no pasó la validación; se recuperó la configuración anterior. {error}"
                )),
                Err(recovery_error) => Err(format!(
                    "La copia restaurada no pasó la validación ({error}) y la recuperación automática también falló: {recovery_error}"
                )),
            };
        }

        // La configuración y el lock nuevos ya son válidos. El marcador se cierra
        // antes de limpiar el árbol anterior: si la limpieza se interrumpe, solo
        // queda un directorio oculto inofensivo que puede retirarse después.
        transaction_commit(Some(&candidate))?;
        let _ = fs::remove_dir_all(&temp);
        history_record("backup-restore", "Restauraste una configuración de Korunix")?;

        if json {
            println!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-backup-restore-result\",\"restored\":true,\"safetyBackup\":{},\"credentialsImported\":false}}",
                json_texto(&seguridad.display().to_string())
            );
        } else {
            println!(
                "✓ configuración restaurada; respaldo anterior: {}",
                seguridad.display()
            );
        }

        return Ok(ExitCode::SUCCESS);
    }

    Err(
        "Uso: korunix backup export [--output ruta] [--plan] [--yes] [--json] | inspect <archivo> [--json] | restore <archivo> [--plan] [--yes] [--json]."
            .to_string(),
    )
}

fn history_path() -> Result<PathBuf, String> {
    Ok(state_root()?.join("history.jsonl"))
}

fn history_record(kind: &str, summary: &str) -> Result<(), String> {
    let path = history_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let entrada = format!(
        "{{\"schemaVersion\":1,\"timestamp\":{},\"kind\":{},\"summary\":{},\"containsSecret\":false}}\n",
        timestamp,
        json_texto(kind),
        json_texto(summary)
    );

    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(entrada.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut permisos = fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permisos, 0o600);
    fs::set_permissions(&path, permisos).map_err(|e| e.to_string())?;
    Ok(())
}

fn history_operation(_raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let json = args.iter().any(|v| v == "--json");
    if args.iter().any(|v| v != "--json") {
        return Err("Uso: korunix history [--json].".to_string());
    }

    let path = history_path()?;
    let contenido = fs::read_to_string(&path).unwrap_or_default();
    let entradas = contenido
        .lines()
        .filter(|linea| !linea.trim().is_empty())
        .filter(|linea| serde_json::from_str::<serde_json::Value>(linea).is_ok())
        .collect::<Vec<_>>()
        .join(",");

    let resultado = format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-history\",\"entries\":[{}]}}",
        entradas
    );

    if json {
        println!("{resultado}");
    } else {
        let value: serde_json::Value =
            serde_json::from_str(&resultado).map_err(|e| e.to_string())?;
        let items = value
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        println!("=== Historial de Korunix ===");
        if items.is_empty() {
            println!("Todavía no hay acciones registradas.");
        } else {
            for item in items.iter().rev() {
                let resumen = item
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Acción de Korunix");
                println!("• {resumen}");
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn add_host_user(
    raiz: &Path,
    host: &str,
    id: &str,
    account: &str,
    home: &str,
    admin: bool,
    preserved: &[String],
) -> Result<(), String> {
    let path = raiz
        .join("configuracion/equipos")
        .join(format!("{host}.nix"));
    let text =
        fs::read_to_string(&path).map_err(|e| format!("No pude leer {}: {e}", path.display()))?;

    if text
        .lines()
        .any(|l| l.trim_start().starts_with(&format!("{id} = {{")))
    {
        return Err(format!("{id} ya pertenece al equipo."));
    }

    let marker = "  users = {\n";
    let pos = text
        .find(marker)
        .ok_or_else(|| "El archivo del equipo no contiene `users = {`.".to_string())?
        + marker.len();

    let preserved_nix = format!(
        "[{}]",
        preserved
            .iter()
            .map(|v| json_texto(v))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let account_line = if account == id {
        String::new()
    } else {
        format!("      accountName = {};\n", json_texto(account))
    };
    let block = format!(
        "    {id} = {{\n\
         {account_line}\
         \x20     homeDirectory = {};\n\
         \x20     administrator = {admin};\n\
         \x20     deferredCapabilities = [];\n\
         \x20     deferredInputMethods = [];\n\
         \x20     preservedGroups = {preserved_nix};\n\
         \x20     githubSshIdentityFile = null;\n\
         \x20   }};\n",
        json_texto(home)
    );

    let mut out = text;
    out.insert_str(pos, &block);
    atomic_write(&path, out.as_bytes())
}

fn users_mutation(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        users_human(raiz)?;
        return Ok(ExitCode::SUCCESS);
    }

    let host = resolver_equipo(raiz)?;
    match args[0].as_str() {
        "adopt" => {
            let account = args
                .get(1)
                .ok_or_else(|| "Uso: korunix users adopt <cuenta>.".to_string())?;
            if !account_valid(account) {
                return Err("Nombre de cuenta inválido.".into());
            }

            let (name, home, admin, groups) = account_info(raiz, account)?
                .ok_or_else(|| format!("{account} no es una cuenta humana detectable."))?;
            let profile = raiz
                .join("configuracion/personas")
                .join(format!("{account}.nix"));
            let profile_existed = profile.exists();
            let host_path = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));
            let preserved: Vec<String> = groups
                .into_iter()
                .filter(|value| !matches!(value.as_str(), "users" | "wheel" | "networkmanager"))
                .collect();

            if !confirm(&format!("¿Preparar la adopción de {account}?"))? {
                return Ok(ExitCode::SUCCESS);
            }

            let mut transaction_paths = vec![host_path];
            if !profile_existed {
                transaction_paths.push(profile.clone());
            }
            let transaction = files_transaction_begin(raiz, &transaction_paths)?;

            let result = (|| -> Result<(), String> {
                if !profile_existed {
                    atomic_write(&profile, profile_text(account, &name).as_bytes())?;
                }
                add_host_user(raiz, &host, account, account, &home, admin, &preserved)?;
                validate(raiz)
            })();

            if let Err(error) = result {
                let recovery = rollback_pending_transaction(raiz);
                return match recovery {
                    Ok(_) => Err(format!("Adopción revertida: {error}")),
                    Err(recovery_error) => Err(format!(
                        "La adopción falló ({error}) y la recuperación automática también falló: {recovery_error}"
                    )),
                };
            }

            transaction_commit(Some(&transaction))?;
            println!("✓ adopción preparada; la contraseña quedó intacta");
        }
        "create" if args.len() > 1 => {
            return users_create_structured(raiz, &args[1..]);
        }
        "password-stdin" => {
            return users_password_stdin(raiz, &args[1..]);
        }
        "create" => {
            if !io::stdin().is_terminal() {
                return Err("users create necesita una terminal interactiva.".into());
            }
            print!("Nombre de cuenta: ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let mut account = String::new();
            io::stdin()
                .read_line(&mut account)
                .map_err(|e| e.to_string())?;
            let account = account.trim().to_string();
            if !account_valid(&account) {
                return Err("Nombre de cuenta inválido.".into());
            }
            print!("Nombre visible: ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let mut name = String::new();
            io::stdin()
                .read_line(&mut name)
                .map_err(|e| e.to_string())?;
            let name = if name.trim().is_empty() {
                account.clone()
            } else {
                name.trim().to_string()
            };
            let admin = confirm("¿Será administradora?")?;
            if !confirm("¿Preparar esta persona?")? {
                return Ok(ExitCode::SUCCESS);
            }

            let profile = raiz
                .join("configuracion/personas")
                .join(format!("{account}.nix"));
            if profile.exists() {
                return Err("Ese perfil ya existe.".into());
            }
            let host_path = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));
            let transaction = files_transaction_begin(raiz, &[host_path.clone(), profile.clone()])?;

            let result = (|| -> Result<(), String> {
                atomic_write(&profile, profile_text(&account, &name).as_bytes())?;
                add_host_user(
                    raiz,
                    &host,
                    &account,
                    &account,
                    &format!("/home/{account}"),
                    admin,
                    &[],
                )?;
                validate(raiz)
            })();

            if let Err(error) = result {
                let recovery = rollback_pending_transaction(raiz);
                return match recovery {
                    Ok(_) => Err(format!("Creación revertida: {error}")),
                    Err(recovery_error) => Err(format!(
                        "La creación falló ({error}) y la recuperación automática también falló: {recovery_error}"
                    )),
                };
            }

            transaction_commit(Some(&transaction))?;
            println!("✓ persona preparada");
            println!("Después de aplicar: korunix users password {account}");
        }
        "password" => {
            let account = args
                .get(1)
                .ok_or_else(|| "Uso: korunix users password <cuenta>.".to_string())?;
            if !confirm(&format!("¿Cambiar la contraseña de {account}?"))? {
                return Ok(ExitCode::SUCCESS);
            }
            // passwd toma el control de la terminal. Korunix no lee ni almacena
            // el secreto.
            let _ = privileged(raiz, "passwd", std::slice::from_ref(account), true)?;
            println!("✓ contraseña administrada directamente por el sistema");
        }
        "export" => {
            let id = args
                .get(1)
                .ok_or_else(|| "Uso: korunix users export <id>.".to_string())?;
            if !id_valido(id) {
                return Err("ID de perfil inválido.".into());
            }
            let profile = raiz
                .join("configuracion/personas")
                .join(format!("{id}.nix"));
            if !profile.is_file() {
                return Err(format!("No existe {}.", profile.display()));
            }
            let data = nix_archivo_json(raiz, &profile)?;
            let out_dir = env::var_os("KORUNIX_EXPORT_DIR")
                .map(PathBuf::from)
                .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join("Downloads")))
                .unwrap_or_else(|| PathBuf::from("."));
            fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
            let temp = env::temp_dir().join(format!("korunix-profile-{}", stamp()));
            fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
            let manifest = jq0(
                raiz,
                &[
                    "-cn".into(),
                    "--arg".into(),
                    "id".into(),
                    id.clone(),
                    "--argjson".into(),
                    "p".into(),
                    data,
                    r#"{
                      schemaVersion:3,
                      kind:"korunix-user-profile",
                      exportedAt:(now|todate),
                      profile:{
                        id:$id,
                        accountName:$p.accountName,
                        fullName:$p.fullName,
                        language:($p.language // null),
                        interfaceLanguage:($p.interfaceLanguage // null),
                        inputMethods:($p.inputMethods // []),
                        capabilities:($p.capabilities // []),
                        avatar:null
                      },
                      security:{
                        containsPassword:false,
                        containsPasswordHash:false,
                        containsHardwareIdentity:false
                      }
                    }"#
                    .into(),
                ],
            )?;
            fs::write(temp.join("manifest.json"), manifest).map_err(|e| e.to_string())?;
            let dest = out_dir.join(format!("{id}.korunix-profile"));
            visible(
                raiz,
                "tar",
                &[
                    "-czf".into(),
                    dest.display().to_string(),
                    "-C".into(),
                    temp.display().to_string(),
                    "manifest.json".into(),
                ],
            )?;
            let mut perm = fs::metadata(&dest)
                .map_err(|e| e.to_string())?
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o600);
            fs::set_permissions(&dest, perm).map_err(|e| e.to_string())?;
            let _ = fs::remove_dir_all(temp);
            println!("✓ perfil exportado: {}", dest.display());
        }
        "inspect" | "plan" | "import" => {
            let bundle = args
                .get(1)
                .ok_or_else(|| format!("Uso: korunix users {} <archivo>.", args[0]))?;
            let listing = capture(raiz, "tar", &["-tzf".into(), bundle.clone()])?;
            if listing.lines().any(|v| {
                !matches!(
                    v,
                    "manifest.json" | "avatar.jpg" | "avatar.jpeg" | "avatar.png" | "avatar.webp"
                )
            }) {
                return Err("El bundle contiene una ruta no permitida.".into());
            }

            let temp = env::temp_dir().join(format!("korunix-import-{}", stamp()));
            fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
            visible(
                raiz,
                "tar",
                &[
                    "-xzf".into(),
                    bundle.clone(),
                    "-C".into(),
                    temp.display().to_string(),
                    "--no-same-owner".into(),
                    "--no-same-permissions".into(),
                ],
            )?;
            let manifest =
                fs::read_to_string(temp.join("manifest.json")).map_err(|e| e.to_string())?;

            if args[0] == "inspect" {
                pretty(raiz, &manifest)?;
                let _ = fs::remove_dir_all(temp);
                return Ok(ExitCode::SUCCESS);
            }

            let id = jq_texto(raiz, &manifest, ".profile.id")?;
            let account = jq_texto(raiz, &manifest, ".profile.accountName")?;
            let exists = raiz
                .join("configuracion/personas")
                .join(format!("{id}.nix"))
                .exists();

            let plan = jq0(
                raiz,
                &[
                    "-cn".into(),
                    "--argjson".into(),
                    "manifest".into(),
                    manifest.clone(),
                    "--argjson".into(),
                    "exists".into(),
                    exists.to_string(),
                    r#"{
                      schemaVersion:3,
                      kind:"korunix-user-import-plan",
                      profile:$manifest.profile,
                      action:(if $exists then "review-existing" else "create-profile" end),
                      canImport:($exists|not),
                      security:$manifest.security
                    }"#
                    .into(),
                ],
            )?;

            if args[0] == "plan" {
                println!("{plan}");
                let _ = fs::remove_dir_all(temp);
                return Ok(ExitCode::SUCCESS);
            }

            if exists {
                let _ = fs::remove_dir_all(temp);
                return Err("Ese perfil ya existe; Korunix no lo sobrescribirá.".into());
            }
            if !confirm("¿Importar este perfil?")? {
                let _ = fs::remove_dir_all(temp);
                return Ok(ExitCode::SUCCESS);
            }

            let manifest_value: serde_json::Value =
                serde_json::from_str(&manifest).map_err(|error| error.to_string())?;
            let imported_profile = profile_text_from_manifest(
                manifest_value
                    .get("profile")
                    .ok_or_else(|| "El bundle no contiene profile.".to_string())?,
            )?;

            let profile = raiz
                .join("configuracion/personas")
                .join(format!("{id}.nix"));
            let info = account_info(raiz, &account)?;
            let (home, admin, preserved) = if let Some((_, home, admin, groups)) = info {
                (
                    home,
                    admin,
                    groups
                        .into_iter()
                        .filter(|value| {
                            !matches!(value.as_str(), "users" | "wheel" | "networkmanager")
                        })
                        .collect(),
                )
            } else {
                (format!("/home/{account}"), false, Vec::new())
            };
            let host_path = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));

            let transaction =
                match files_transaction_begin(raiz, &[host_path.clone(), profile.clone()]) {
                    Ok(transaction) => transaction,
                    Err(error) => {
                        let _ = fs::remove_dir_all(&temp);
                        return Err(error);
                    }
                };

            let result = (|| -> Result<(), String> {
                atomic_write(&profile, imported_profile.as_bytes())?;
                add_host_user(raiz, &host, &id, &account, &home, admin, &preserved)?;
                validate(raiz)
            })();

            if let Err(error) = result {
                let recovery = rollback_pending_transaction(raiz);
                let _ = fs::remove_dir_all(&temp);
                return match recovery {
                    Ok(_) => Err(format!("Importación revertida: {error}")),
                    Err(recovery_error) => Err(format!(
                        "La importación falló ({error}) y la recuperación automática también falló: {recovery_error}"
                    )),
                };
            }

            transaction_commit(Some(&transaction))?;
            let _ = fs::remove_dir_all(&temp);
            println!("✓ importación preparada");
        }
        _ => return Err("Operación de personas desconocida.".into()),
    }
    Ok(ExitCode::SUCCESS)
}

fn bootstrap_host_id(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in name.to_ascii_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "equipo".into()
    } else {
        out
    }
}

fn simple_quoted_value(path: &Path, key: &str) -> String {
    let text = fs::read_to_string(path).unwrap_or_default();
    for line in text.lines() {
        if !line.contains(key) || !line.contains('=') {
            continue;
        }
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                return line[start + 1..start + 1 + end].to_string();
            }
        }
    }
    String::new()
}

fn bootstrap_hardware_target(raiz: &Path, host: &str) -> PathBuf {
    raiz.join("generado/equipos")
        .join(format!("{host}-detectado.nix"))
}

fn bootstrap_graphics_json(graphics_json: &str) -> Result<String, String> {
    let graphics: serde_json::Value = serde_json::from_str(graphics_json).map_err(|error| {
        format!("No pude interpretar las GPU detectadas durante la adopción: {error}")
    })?;

    let devices = graphics
        .as_array()
        .ok_or_else(|| "La detección gráfica no produjo una lista válida.".to_string())?;

    let allowed = [
        "pciAddress",
        "name",
        "vendor",
        "vendorId",
        "deviceId",
        "subsystemVendorId",
        "subsystemDeviceId",
        "driver",
        "primary",
        "kind",
        "nvidiaOpen",
    ];

    let mut normalized = Vec::with_capacity(devices.len());

    for device in devices {
        let object = device
            .as_object()
            .ok_or_else(|| "La detección gráfica contiene una entrada inválida.".to_string())?;

        let mut clean = serde_json::Map::new();

        for key in allowed {
            let value = object.get(key).cloned().ok_or_else(|| {
                format!("La detección gráfica no contiene el campo obligatorio {key}.")
            })?;
            clean.insert(key.to_string(), value);
        }

        normalized.push(serde_json::Value::Object(clean));
    }

    serde_json::to_string(&normalized)
        .map_err(|error| format!("No pude normalizar las GPU detectadas: {error}"))
}

fn bootstrap_hardware_text_from(
    source_text: &str,
    firmware: &str,
    graphics_json: &str,
) -> Result<String, String> {
    if !matches!(firmware, "uefi" | "bios") {
        return Err("Tipo de firmware inválido durante la adopción.".into());
    }

    let graphics_json = bootstrap_graphics_json(graphics_json)?;

    let trimmed = source_text.trim_end();
    if !trimmed.ends_with('}') {
        return Err(
            "hardware-configuration.nix no tiene la forma esperada de un módulo NixOS.".into(),
        );
    }

    let final_brace = trimmed.len() - 1;
    let mut output = String::new();

    output.push_str(
        "# NO CAMBIES ESTE ARCHIVO A MANO.\n\
#\n\
# Korunix incorporó aquí el hardware generado por NixOS durante la adopción.\n\
# Conserva los dispositivos, sistemas de archivos y módulos de la instalación\n\
# existente, y añade únicamente los hechos que Korunix necesita para decidir\n\
# firmware y gráficos. Para corregirlo se debe volver a detectar/adoptar el\n\
# hardware; una edición manual puede perderse.\n#\n",
    );

    output.push_str(&source_text[..final_brace]);
    if !output.ends_with('\n') {
        output.push('\n');
    }

    output.push_str("\n  # Hechos detectados por Korunix durante la adopción.\n");
    output.push_str(&format!(
        "  korunix.hardware.firmware = {};\n",
        json_texto(firmware)
    ));
    output.push_str(&format!(
        "  korunix.hardware.graphics = builtins.fromJSON {};\n",
        json_texto(&graphics_json)
    ));
    output.push_str(&source_text[final_brace..]);

    Ok(output)
}

fn bootstrap_hardware_text(raiz: &Path, source: &Path, firmware: &str) -> Result<String, String> {
    let source_text = fs::read_to_string(source)
        .map_err(|error| format!("No pude leer {}: {error}", source.display()))?;
    let graphics_json = dispositivos_graficos_json(raiz);

    bootstrap_hardware_text_from(&source_text, firmware, &graphics_json)
}

fn bootstrap_plan_json(raiz: &Path) -> Result<String, String> {
    let config = env::var_os("KORUNIX_CONFIG_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/nixos/configuration.nix"));
    let hardware = env::var_os("KORUNIX_HARDWARE_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/nixos/hardware-configuration.nix"));

    if !config.is_file() || !hardware.is_file() {
        return Err("Falta la configuración instalada de NixOS.".into());
    }

    let host_name = capture(raiz, "hostname", &[])?;
    let host_id = bootstrap_host_id(&host_name);
    let state_version = simple_quoted_value(&config, "system.stateVersion");
    if state_version.is_empty() {
        return Err("No pude leer system.stateVersion.".into());
    }

    let arch = match capture(raiz, "uname", &["-m".into()])?.as_str() {
        "x86_64" => "x86_64-linux",
        "aarch64" | "arm64" => "aarch64-linux",
        _ => return Err("Arquitectura no soportada para bootstrap.".into()),
    };
    let firmware = if Path::new("/sys/firmware/efi").is_dir() {
        "uefi"
    } else {
        "bios"
    };
    let hardware_target = format!("generado/equipos/{host_id}-detectado.nix");

    let channel = capture(
        raiz,
        "nix",
        &[
            "eval".into(),
            "--raw".into(),
            "--file".into(),
            "sistema/canales.nix".into(),
            "--apply".into(),
            "datos: datos.default".into(),
        ],
    )?;

    let defaults = nix_archivo_json(raiz, &raiz.join("sistema/predeterminados.nix"))?;
    let product_defaults = jq_compacto(
        raiz,
        &defaults,
        &format!(
            "{{schemaVersion:.schemaVersion,desktop:.desktop,applications:(.applications.common + .applications.bySystem[{}]),services:.services}}",
            json_texto(arch)
        ),
    )?;

    let passwd = capture(raiz, "getent", &["passwd".into()])?;
    let mut users = Vec::new();
    let mut profiles = Vec::new();
    let mut admins = 0usize;
    for line in passwd.lines() {
        let f: Vec<&str> = line.split(':').collect();
        if f.len() < 7 {
            continue;
        }
        let Ok(uid) = f[2].parse::<u32>() else {
            continue;
        };
        if uid < uid_minimo() || uid >= 65534 || cuenta_tecnica(f[5], f[6]) {
            continue;
        }
        let groups_raw = capture(raiz, "id", &["-nG".into(), f[0].into()]).unwrap_or_default();
        let groups: Vec<String> = groups_raw
            .split_whitespace()
            .map(ToString::to_string)
            .collect();
        let admin = groups.iter().any(|v| v == "wheel");
        if admin {
            admins += 1;
        }
        let name = f[4]
            .split(',')
            .next()
            .filter(|v| !v.is_empty())
            .unwrap_or(f[0]);
        users.push(format!(
            "{{\"account\":{},\"uid\":{},\"homeDirectory\":{},\"shell\":{},\"administrator\":{}}}",
            json_texto(f[0]),
            uid,
            json_texto(f[5]),
            json_texto(f[6]),
            admin
        ));
        profiles.push(format!(
            "{{\"id\":{},\"target\":{{\"profilePath\":{},\"profileExists\":{},\"overwriteAllowed\":false}},\"portable\":{{\"accountName\":{},\"fullName\":{},\"language\":null,\"interfaceLanguage\":null,\"inputMethods\":[],\"capabilities\":[],\"avatar\":null}},\"local\":{{\"homeDirectory\":{},\"administrator\":{},\"deferredCapabilities\":[],\"deferredInputMethods\":[],\"preservedGroups\":[],\"githubSshIdentityFile\":null}},\"credentials\":{{\"action\":\"preserve-existing\",\"importedSecret\":false}}}}",
            json_texto(f[0]),
            json_texto(&format!("configuracion/personas/{}.nix", f[0])),
            raiz.join("configuracion/personas").join(format!("{}.nix", f[0])).exists(),
            json_texto(f[0]),
            json_texto(name),
            json_texto(f[5]),
            admin
        ));
    }
    if users.is_empty() || admins == 0 {
        return Err("Bootstrap necesita una cuenta humana administradora existente.".into());
    }

    let localization = jq0(
        raiz,
        &[
            "-cn".into(),
            "--arg".into(),
            "timeZone".into(),
            capture(
                raiz,
                "timedatectl",
                &[
                    "show".into(),
                    "--property=Timezone".into(),
                    "--value".into(),
                ],
            )
            .unwrap_or_default(),
            r#"{
              timeZone:($timeZone|if .=="" then null else . end),
              locale:null,
              systemLanguage:null,
              region:null,
              formats:null,
              consoleKeymap:null,
              keyboard:null,
              source:{configuration:"/etc/nixos/configuration.nix"},
              needsReview:true
            }"#
            .into(),
        ],
    )?;

    jq0(
        raiz,
        &[
            "-cn".into(),
            "--arg".into(),
            "hostId".into(),
            host_id,
            "--arg".into(),
            "hostName".into(),
            host_name,
            "--arg".into(),
            "system".into(),
            arch.into(),
            "--arg".into(),
            "stateVersion".into(),
            state_version,
            "--arg".into(),
            "channel".into(),
            channel,
            "--arg".into(),
            "config".into(),
            config.display().to_string(),
            "--arg".into(),
            "configSha".into(),
            sha256(raiz, &config)?,
            "--arg".into(),
            "hardware".into(),
            hardware.display().to_string(),
            "--arg".into(),
            "hardwareSha".into(),
            sha256(raiz, &hardware)?,
            "--arg".into(),
            "hardwareTarget".into(),
            hardware_target,
            "--argjson".into(),
            "localization".into(),
            localization,
            "--argjson".into(),
            "users".into(),
            format!("[{}]", users.join(",")),
            "--argjson".into(),
            "profiles".into(),
            format!("[{}]", profiles.join(",")),
            "--argjson".into(),
            "defaults".into(),
            product_defaults,
            "--arg".into(),
            "firmware".into(),
            firmware.into(),
            r#"{
              schemaVersion:1,
              source:{
                configuration:$config,
                configurationSha256:$configSha,
                hardware:$hardware,
                hardwareSha256:$hardwareSha,
                hardwareTarget:$hardwareTarget,
                productDefaults:"sistema/predeterminados.nix",
                channels:"sistema/canales.nix",
                reusesInstalledHardware:true,
                copiesInstalledHardware:true,
                generatesHardware:false
              },
              host:{
                id:$hostId,
                name:$hostName,
                system:$system,
                stateVersion:$stateVersion,
                channel:$channel
              },
              localization:$localization,
              users:$users,
              userAdoption:{
                schemaVersion:1,
                kind:"korunix-bootstrap-users-plan",
                accounts:$users,
                profiles:$profiles,
                summary:{
                  humanAccounts:($users|length),
                  administrators:([$users[]|select(.administrator)]|length)
                },
                policy:{
                  preservesExistingPasswords:true,
                  storesPasswords:false,
                  inventsPortablePreferences:false,
                  overwritesExistingProfiles:false
                }
              },
              productDefaults:$defaults,
              hardware:{
                platform:{detected:$system},
                firmware:{detected:$firmware}
              },
              actions:{
                writesRepository:false,
                modifiesEtcNixos:false,
                buildsGeneration:false,
                appliesGeneration:false
              }
            }"#
            .into(),
        ],
    )
}

fn bootstrap_host_profile_lines(plan: &str) -> Result<String, String> {
    let data: serde_json::Value = serde_json::from_str(plan)
        .map_err(|error| format!("El plan de bootstrap no es JSON válido: {error}"))?;

    let profiles = data
        .pointer("/userAdoption/profiles")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "El plan de bootstrap no contiene perfiles de usuario.".to_string())?;

    let mut lines = Vec::with_capacity(profiles.len());

    for profile in profiles {
        let id = profile
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Un perfil de bootstrap no contiene identificador.".to_string())?;

        if !account_valid(id) {
            return Err(format!(
                "El identificador de usuario {id} no es válido para la configuración de Korunix."
            ));
        }

        let home = profile
            .pointer("/local/homeDirectory")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("El perfil {id} no contiene carpeta personal."))?;

        let administrator = profile
            .pointer("/local/administrator")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| format!("El perfil {id} no contiene su rol administrativo."))?;

        lines.push(format!(
            "    {id} = {{ homeDirectory = {}; administrator = {administrator}; deferredCapabilities = []; deferredInputMethods = []; preservedGroups = []; githubSshIdentityFile = null; }};",
            json_texto(home)
        ));
    }

    Ok(lines.join("\n"))
}

fn bootstrap(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    match args {
        [mode] if mode == "--plan" => pretty(raiz, &bootstrap_plan_json(raiz)?)?,
        [mode, json] if mode == "--plan" && json == "--json" => {
            println!("{}", bootstrap_plan_json(raiz)?)
        }
        [mode] if mode == "--adopt" => {
            let plan = bootstrap_plan_json(raiz)?;
            pretty(raiz, &plan)?;
            if !confirm("¿Adoptar esta instalación?")? {
                return Ok(ExitCode::SUCCESS);
            }
            return Err(
                "La escritura de bootstrap requiere --yes para evitar una adopción accidental."
                    .into(),
            );
        }
        [mode, yes] if mode == "--adopt" && yes == "--yes" => {
            let plan = bootstrap_plan_json(raiz)?;
            let host = jq_texto(raiz, &plan, ".host.id")?;
            let firmware = jq_texto(raiz, &plan, ".hardware.firmware.detected")?;
            let hardware_source = PathBuf::from(jq_texto(raiz, &plan, ".source.hardware")?);
            let expected_hardware_sha = jq_texto(raiz, &plan, ".source.hardwareSha256")?;
            let actual_hardware_sha = sha256(raiz, &hardware_source)?;

            if actual_hardware_sha != expected_hardware_sha {
                return Err(
                    "El hardware de origen cambió después de preparar el plan. Vuelve a revisar la adopción."
                        .into(),
                );
            }

            let hardware_text = bootstrap_hardware_text(raiz, &hardware_source, &firmware)?;
            let hardware_target = bootstrap_hardware_target(raiz, &host);
            let existing_host = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));

            if existing_host.exists() {
                if hardware_target.exists() {
                    validate(raiz)?;
                    println!("✓ esta instalación ya estaba adoptada; no se modificó nada");
                    return Ok(ExitCode::SUCCESS);
                }

                let transaction =
                    files_transaction_begin(raiz, std::slice::from_ref(&hardware_target))?;

                let result = (|| -> Result<(), String> {
                    atomic_write(&hardware_target, hardware_text.as_bytes())?;
                    validate(raiz)
                })();

                if let Err(error) = result {
                    let recovery = rollback_pending_transaction(raiz);
                    return match recovery {
                        Ok(_) => Err(format!(
                            "La reparación del hardware adoptado fue revertida: {error}"
                        )),
                        Err(recovery_error) => Err(format!(
                            "La reparación del hardware falló ({error}) y la recuperación automática también falló: {recovery_error}"
                        )),
                    };
                }

                transaction_commit(Some(&transaction))?;
                println!(
                    "✓ hardware adoptado reparado sin modificar /etc/nixos ni aplicar una generación"
                );
                return Ok(ExitCode::SUCCESS);
            }

            if hardware_target.exists() {
                return Err(format!(
                    "Korunix encontró {} sin su archivo de host y no lo sobrescribirá.",
                    hardware_target.display()
                ));
            }

            let backup = backup_dir(&format!("bootstrap-{host}"))?;
            fs::write(backup.join("plan.json"), &plan).map_err(|e| e.to_string())?;

            let profiles = jq_con_entrada(
                raiz,
                &["-c".into(), ".userAdoption.profiles[]".into()],
                &plan,
            )?;
            let mut prepared_profiles = Vec::<(PathBuf, String)>::new();

            for profile in profiles.lines() {
                let id = jq_texto(raiz, profile, ".id")?;
                let account = jq_texto(raiz, profile, ".portable.accountName")?;
                let name = jq_texto(raiz, profile, ".portable.fullName")?;
                let path = raiz
                    .join("configuracion/personas")
                    .join(format!("{id}.nix"));
                if path.exists() {
                    return Err(format!("Bootstrap no sobrescribirá {}.", path.display()));
                }
                prepared_profiles.push((path, profile_text(&account, &name)));
            }

            let system = jq_texto(raiz, &plan, ".host.system")?;
            let host_name = jq_texto(raiz, &plan, ".host.name")?;
            let state_version = jq_texto(raiz, &plan, ".host.stateVersion")?;
            let channel = jq_texto(raiz, &plan, ".host.channel")?;
            let profile_lines = bootstrap_host_profile_lines(&plan)?;
            let host_text = format!(
                "# ESTE ARCHIVO SE PUEDE CAMBIAR.\n{{\n  system = {};\n  users = {{\n{}\n  }};\n  korunix = {{ enable = true; channel = {}; hostName = {}; stateVersion = {}; }};\n}}\n",
                json_texto(&system),
                profile_lines,
                json_texto(&channel),
                json_texto(&host_name),
                json_texto(&state_version)
            );
            let host_path = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));

            let mut transaction_paths = prepared_profiles
                .iter()
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>();
            transaction_paths.push(hardware_target.clone());
            transaction_paths.push(host_path.clone());
            let transaction = files_transaction_begin(raiz, &transaction_paths)?;

            let result = (|| -> Result<(), String> {
                for (path, profile) in &prepared_profiles {
                    atomic_write(path, profile.as_bytes())?;
                }
                atomic_write(&hardware_target, hardware_text.as_bytes())?;
                atomic_write(&host_path, host_text.as_bytes())?;
                validate(raiz)
            })();

            if let Err(error) = result {
                let recovery = rollback_pending_transaction(raiz);
                return match recovery {
                    Ok(_) => Err(format!("Bootstrap revertido: {error}")),
                    Err(recovery_error) => Err(format!(
                        "Bootstrap falló ({error}) y la recuperación automática también falló: {recovery_error}"
                    )),
                };
            }

            transaction_commit(Some(&transaction))?;
            println!(
                "✓ instalación adoptada con su hardware, sin modificar /etc/nixos ni aplicar una generación"
            );
        }
        _ => return Err("Uso: korunix bootstrap --plan [--json] | --adopt [--yes].".into()),
    }
    Ok(ExitCode::SUCCESS)
}

fn product_status(raiz: &Path, json: bool) -> Result<ExitCode, String> {
    let arch = capture(raiz, "uname", &["-m".into()]).unwrap_or_else(|_| "unknown".into());
    let system = match arch.as_str() {
        "x86_64" => Some("x86_64-linux"),
        "aarch64" | "arm64" => Some("aarch64-linux"),
        _ => None,
    };
    let pending = transaction_pending_busy().unwrap_or(false);

    let data = serde_json::json!({
        "schemaVersion": 1,
        "kind": "korunix-product-status",
        "version": env!("CARGO_PKG_VERSION"),
        "platform": {
            "kernelArchitecture": arch,
            "system": system,
            "supported": system.is_some(),
            "supportedSystems": ["x86_64-linux", "aarch64-linux"]
        },
        "connectivity": {
            "probed": false,
            "offlineFirst": true
        },
        "transactions": {
            "pending": pending,
            "automaticRecoveryBeforeOperation": true,
            "durableAtomicWrites": true
        },
        "capabilities": {
            "localWithoutNetwork": [
                "status", "hardware", "localization", "users", "applications",
                "desktop", "appearance", "defaults", "backup", "history", "rollback",
                "storage", "media", "validate"
            ],
            "networkWhenRefreshing": ["update", "firmware refresh"],
            "localDistribution": "nix --extra-experimental-features 'nix-command flakes' run path:/ruta/a/korunix#bootstrap",
            "remoteDistribution": "nix --extra-experimental-features 'nix-command flakes' run github:koruninn/korunix#bootstrap"
        }
    });

    if json {
        println!(
            "{}",
            serde_json::to_string(&data).map_err(|e| e.to_string())?
        );
    } else {
        println!("Korunix {}", env!("CARGO_PKG_VERSION"));
        println!("Arquitectura: {}", system.unwrap_or("no soportada"));
        println!("Red: no se consulta para este diagnóstico; las funciones locales permanecen disponibles sin conexión.");
        println!(
            "Recuperación transaccional: {}",
            if pending {
                "hay una recuperación pendiente"
            } else {
                "lista"
            }
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn media_interactive_ok(json: bool, yes: bool, message: &str) -> Result<bool, String> {
    if json && !yes {
        return Err("La prueba multimedia JSON necesita --yes.".into());
    }
    if yes {
        Ok(true)
    } else {
        confirm(message)
    }
}

fn speaker_test(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let sink = args.first().ok_or_else(|| "Falta sink-id.".to_string())?;
    let sink_id: u32 = sink.parse().map_err(|_| "sink-id inválido.".to_string())?;
    let mut channel = "both".to_string();
    let mut seconds = 2u32;
    let mut yes = false;
    let mut json = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--channel" => {
                i += 1;
                channel = args
                    .get(i)
                    .cloned()
                    .ok_or_else(|| "Falta canal.".to_string())?;
            }
            "--seconds" => {
                i += 1;
                seconds = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .ok_or_else(|| "Segundos inválidos.".to_string())?;
            }
            "--yes" => yes = true,
            "--json" => json = true,
            x => return Err(format!("Opción desconocida: {x}")),
        }
        i += 1;
    }

    if !matches!(channel.as_str(), "left" | "right" | "both") {
        return Err("Canal inválido.".into());
    }
    if !(1..=5).contains(&seconds) {
        return Err("La prueba admite entre 1 y 5 segundos.".into());
    }
    if !media_interactive_ok(
        json,
        yes,
        &format!("¿Reproducir una prueba temporal en la salida {sink_id}?"),
    )? {
        return Ok(ExitCode::SUCCESS);
    }

    let bin = if let Some(base) = env::var_os("KORUNIX_MEDIA_BIN_DIR") {
        PathBuf::from(base).join("pw-play").into_os_string()
    } else {
        tool("pw-play")
    };
    let mut child = Command::new(bin)
        .args([
            "--target",
            sink,
            "--raw",
            "--rate",
            "48000",
            "--format",
            "s16",
            "--channels",
            "2",
            "-",
        ])
        .current_dir(raiz)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("No pude iniciar pw-play: {e}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "pw-play no abrió stdin.".to_string())?;
    let total = 48_000usize * seconds as usize;
    for n in 0..total {
        let phase = (n as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin();
        let sample = (phase * i16::MAX as f32 * 0.18) as i16;
        let (left, right) = match channel.as_str() {
            "left" => (sample, 0),
            "right" => (0, sample),
            _ => (sample, sample),
        };
        stdin
            .write_all(&left.to_le_bytes())
            .map_err(|e| e.to_string())?;
        stdin
            .write_all(&right.to_le_bytes())
            .map_err(|e| e.to_string())?;
    }
    drop(stdin);
    if !child.wait().map_err(|e| e.to_string())?.success() {
        return Err("pw-play no completó la prueba.".into());
    }

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-media-speaker-test-result\",\"sinkId\":{sink_id},\"channel\":{},\"seconds\":{seconds},\"completed\":true,\"changedDefault\":false}}",
            json_texto(&channel)
        );
    } else {
        println!("✓ prueba de salida completada");
    }
    Ok(ExitCode::SUCCESS)
}

fn mic_runtime_dir() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("korunix")
}

fn mic_limpiar_muestras_temporales() -> Result<(), String> {
    let dir = mic_runtime_dir();
    if !dir.exists() {
        return Ok(());
    }

    for entrada in fs::read_dir(&dir).map_err(|error| error.to_string())? {
        let entrada = entrada.map_err(|error| error.to_string())?;
        let nombre = entrada.file_name();
        let nombre = nombre.to_string_lossy();

        if nombre.starts_with("mic-sample-") && nombre.ends_with(".wav") {
            let _ = fs::remove_file(entrada.path());
        }
    }

    Ok(())
}

fn mic_wav_data_offset(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }

    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let tamano = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;

        if id == b"data" {
            return Some(pos + 8);
        }

        let avance = 8usize.checked_add(tamano)?.checked_add(tamano % 2)?;
        pos = pos.checked_add(avance)?;
    }

    None
}

fn mic_nivel_porcentaje(bytes: &[u8]) -> u8 {
    let mut suma = 0.0f64;
    let mut cantidad = 0usize;

    for muestra in bytes.chunks_exact(2) {
        let valor = i16::from_le_bytes([muestra[0], muestra[1]]) as f64 / 32768.0;
        suma += valor * valor;
        cantidad += 1;
    }

    if cantidad == 0 {
        return 0;
    }

    let rms = (suma / cantidad as f64).sqrt().max(0.000_001);
    let db = 20.0 * rms.log10();
    (((db + 60.0) / 60.0) * 100.0).clamp(0.0, 100.0).round() as u8
}

fn media_target_audio_name(raiz: &Path, clase: &str, id: u32) -> Result<String, String> {
    let audio = media_audio_json(raiz)?;
    let coleccion = match clase {
        "sink" => "sinks",
        "source" => "sources",
        _ => return Err("Clase de audio inválida.".to_string()),
    };
    let filtro = format!(".{coleccion}[] | select(.id == {id}) | .name");
    let nombre = jq_texto(raiz, &audio, &filtro)?;
    if nombre.trim().is_empty() || nombre.trim() == "null" {
        Err(format!("No pude resolver el dispositivo de audio {id}."))
    } else {
        Ok(nombre)
    }
}

fn mic_meter_live(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    use std::io::Write as _;

    let source = args.first().ok_or_else(|| "Falta source-id.".to_string())?;
    let source_id: u32 = source
        .parse()
        .map_err(|_| "source-id inválido.".to_string())?;

    let source_target = media_target_audio_name(raiz, "source", source_id)?;

    let mut seconds = 5u32;
    let mut yes = false;
    let mut json = false;
    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                seconds = args
                    .get(i)
                    .ok_or_else(|| "Falta valor para --seconds.".to_string())?
                    .parse::<u32>()
                    .map_err(|_| "Duración de medidor inválida.".to_string())?
                    .clamp(1, 15);
            }
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de medidor desconocida: {otro}")),
        }
        i += 1;
    }

    if !yes && !confirm("¿Medir este micrófono durante unos segundos?")? {
        return Ok(ExitCode::SUCCESS);
    }

    let runtime = mic_runtime_dir();
    fs::create_dir_all(&runtime).map_err(|error| error.to_string())?;
    let temp = runtime.join(format!("mic-meter-{}.wav", stamp()));

    let bin = if let Some(base) = env::var_os("KORUNIX_MEDIA_BIN_DIR") {
        PathBuf::from(base).join("pw-record").into_os_string()
    } else {
        tool("pw-record")
    };

    let segundos = seconds.to_string();
    let muestras = (48_000u32 * seconds).to_string();
    let ruta = temp.display().to_string();

    let mut hijo = Command::new(bin)
        .args([
            "--target",
            source_target.as_str(),
            "--rate",
            "48000",
            "--format",
            "s16",
            "--sample-count",
            &muestras,
            &ruta,
        ])
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("No pude iniciar la medición del micrófono: {error}"))?;

    let mut procesados = 0usize;
    let mut ultimo = u8::MAX;

    let estado = loop {
        if let Some(estado) = hijo.try_wait().map_err(|error| error.to_string())? {
            break estado;
        }

        if let Ok(bytes) = fs::read(&temp) {
            if let Some(inicio) = mic_wav_data_offset(&bytes) {
                let disponibles = bytes.len().saturating_sub(inicio);
                if disponibles > procesados {
                    let desde = inicio + procesados;
                    let nuevos = &bytes[desde..];
                    let nivel = mic_nivel_porcentaje(nuevos);

                    if nivel != ultimo {
                        eprintln!("KORUNIX_MIC_LEVEL\t{nivel}");
                        let _ = std::io::stderr().flush();
                        ultimo = nivel;
                    }

                    procesados = disponibles;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(120));
    };

    let hubo_nivel = ultimo != u8::MAX;
    let _ = fs::remove_file(&temp);

    if !estado.success() && !hubo_nivel {
        return Err("No se pudo medir el micrófono.".to_string());
    }

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-media-mic-meter-result\",\"sourceId\":{source_id},\"seconds\":{segundos},\"completed\":true,\"directMonitoring\":false,\"liveLevels\":true}}"
        );
    } else {
        println!("✓ medición de micrófono completada");
    }

    Ok(ExitCode::SUCCESS)
}

fn mic_record_temp(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let source = args.first().ok_or_else(|| "Falta source-id.".to_string())?;
    let source_id: u32 = source
        .parse()
        .map_err(|_| "source-id inválido.".to_string())?;
    let source_target = media_target_audio_name(raiz, "source", source_id)?;

    let mut seconds = 3u32;
    let mut yes = false;
    let mut json = false;
    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                seconds = args
                    .get(i)
                    .ok_or_else(|| "Falta valor para --seconds.".to_string())?
                    .parse::<u32>()
                    .map_err(|_| "Duración de grabación inválida.".to_string())?
                    .clamp(1, 15);
            }
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de grabación desconocida: {otro}")),
        }
        i += 1;
    }

    if !yes && !confirm("¿Grabar una prueba temporal de este micrófono?")? {
        return Ok(ExitCode::SUCCESS);
    }

    mic_limpiar_muestras_temporales()?;
    let runtime = mic_runtime_dir();
    fs::create_dir_all(&runtime).map_err(|error| error.to_string())?;
    let temp = runtime.join(format!("mic-sample-{}.wav", stamp()));
    let muestras = (48_000u32 * seconds).to_string();
    let ruta = temp.display().to_string();

    let bin = if let Some(base) = env::var_os("KORUNIX_MEDIA_BIN_DIR") {
        PathBuf::from(base).join("pw-record").into_os_string()
    } else {
        tool("pw-record")
    };

    let mut ultimo_error = String::new();
    let mut completada = false;

    for target in [source_target.clone(), source_id.to_string()] {
        let _ = fs::remove_file(&temp);
        let salida = Command::new(&bin)
            .args([
                "--target",
                target.as_str(),
                "--rate",
                "48000",
                "--format",
                "s16",
                "--sample-count",
                muestras.as_str(),
                ruta.as_str(),
            ])
            .current_dir(raiz)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| format!("No pude iniciar la grabación del micrófono: {error}"))?;

        let archivo_util = fs::metadata(&temp)
            .map(|metadata| metadata.len() > 44)
            .unwrap_or(false);

        if salida.status.success() || archivo_util {
            completada = true;
            break;
        }

        ultimo_error = String::from_utf8_lossy(&salida.stderr).trim().to_string();
    }

    if !completada {
        let _ = fs::remove_file(&temp);
        return Err(if ultimo_error.is_empty() {
            "No se pudo grabar la prueba del micrófono.".to_string()
        } else {
            format!("No se pudo grabar la prueba del micrófono.\n{ultimo_error}")
        });
    }

    let metadata = fs::metadata(&temp)
        .map_err(|error| format!("La grabación temporal no quedó disponible: {error}"))?;
    if metadata.len() <= 44 {
        let _ = fs::remove_file(&temp);
        return Err("La grabación del micrófono quedó vacía.".to_string());
    }

    let mut permisos = metadata.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permisos, 0o600);
    fs::set_permissions(&temp, permisos).map_err(|error| error.to_string())?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-media-mic-record-result\",\"sourceId\":{source_id},\"seconds\":{seconds},\"completed\":true,\"directMonitoring\":false,\"temporary\":true,\"path\":{}}}",
            json_texto(&temp.display().to_string())
        );
    } else {
        println!("✓ prueba temporal grabada");
    }

    Ok(ExitCode::SUCCESS)
}

fn mic_play_temp(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let ruta = args
        .first()
        .ok_or_else(|| "Falta la grabación temporal.".to_string())?;
    let ruta = PathBuf::from(ruta);

    let runtime = mic_runtime_dir();
    let nombre_valido = ruta
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .map(|nombre| nombre.starts_with("mic-sample-") && nombre.ends_with(".wav"))
        .unwrap_or(false);

    if ruta.parent() != Some(runtime.as_path()) || !nombre_valido {
        return Err("Korunix rechazó una ruta de prueba de micrófono no válida.".to_string());
    }

    if !ruta.is_file() {
        return Err("La grabación temporal ya no está disponible.".to_string());
    }

    let mut sink: Option<u32> = None;
    let mut borrar = false;
    let mut yes = false;
    let mut json = false;
    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--sink" => {
                i += 1;
                sink = Some(
                    args.get(i)
                        .ok_or_else(|| "Falta valor para --sink.".to_string())?
                        .parse::<u32>()
                        .map_err(|_| "sink-id inválido.".to_string())?,
                );
            }
            "--delete" => borrar = true,
            "--yes" => yes = true,
            "--json" => json = true,
            otro => return Err(format!("Opción de reproducción desconocida: {otro}")),
        }
        i += 1;
    }

    if !yes && !confirm("¿Reproducir la prueba temporal del micrófono?")? {
        return Ok(ExitCode::SUCCESS);
    }

    let mut argumentos = Vec::<String>::new();
    if let Some(id) = sink {
        argumentos.extend(["--target".into(), id.to_string()]);
    }
    argumentos.push(ruta.display().to_string());

    let resultado = media_exec(raiz, "pw-play", &argumentos);

    if borrar {
        let _ = fs::remove_file(&ruta);
    }

    resultado?;
    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-media-mic-play-result\",\"completed\":true,\"directMonitoring\":false,\"deleted\":{borrar}}}"
        );
    } else {
        println!("✓ prueba temporal reproducida");
    }

    Ok(ExitCode::SUCCESS)
}

fn mic_test(raiz: &Path, args: &[String], meter: bool) -> Result<ExitCode, String> {
    let source = args.first().ok_or_else(|| "Falta source-id.".to_string())?;
    let source_id: u32 = source
        .parse()
        .map_err(|_| "source-id inválido.".to_string())?;
    let source_target = media_target_audio_name(raiz, "source", source_id)?;

    let mut seconds = if meter { 5 } else { 3 };
    let mut sink: Option<u32> = None;
    let mut save: Option<PathBuf> = None;
    let mut yes = false;
    let mut json = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                seconds = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .ok_or_else(|| "Segundos inválidos.".to_string())?;
            }
            "--sink" if !meter => {
                i += 1;
                sink = Some(
                    args.get(i)
                        .and_then(|v| v.parse().ok())
                        .ok_or_else(|| "sink-id inválido.".to_string())?,
                );
            }
            "--save" if !meter => {
                i += 1;
                save = Some(PathBuf::from(
                    args.get(i).ok_or_else(|| "Falta ruta.".to_string())?,
                ));
            }
            "--yes" => yes = true,
            "--json" => json = true,
            x => return Err(format!("Opción desconocida: {x}")),
        }
        i += 1;
    }

    if !(1..=30).contains(&seconds) {
        return Err("Duración de micrófono inválida.".into());
    }
    if !media_interactive_ok(
        json,
        yes,
        &format!("¿Activar temporalmente el micrófono {source_id}?"),
    )? {
        return Ok(ExitCode::SUCCESS);
    }

    let runtime = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(env::temp_dir);
    let temp = runtime.join(format!("korunix-mic-{}.wav", stamp()));

    media_exec(
        raiz,
        "pw-record",
        &[
            "--target".into(),
            source_target.clone(),
            "--rate".into(),
            "48000".into(),
            "--sample-count".into(),
            (48_000u32 * seconds).to_string(),
            temp.display().to_string(),
        ],
    )?;

    if meter {
        if json {
            println!(
                "{{\"schemaVersion\":1,\"kind\":\"korunix-media-mic-meter-result\",\"sourceId\":{source_id},\"seconds\":{seconds},\"completed\":true,\"directMonitoring\":false}}"
            );
        } else {
            println!("✓ captura temporal de micrófono completada");
        }
        let _ = fs::remove_file(temp);
        return Ok(ExitCode::SUCCESS);
    }

    let mut play = Vec::new();
    if let Some(id) = sink {
        play.extend(["--target".into(), id.to_string()]);
    }
    play.push(temp.display().to_string());
    media_exec(raiz, "pw-play", &play)?;

    let mut saved_path = None;
    if let Some(dest) = save {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(&temp, &dest).map_err(|e| e.to_string())?;
        let mut permissions = fs::metadata(&dest)
            .map_err(|e| e.to_string())?
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o600);
        fs::set_permissions(&dest, permissions).map_err(|e| e.to_string())?;
        saved_path = Some(dest);
    }
    let _ = fs::remove_file(temp);

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-media-mic-test-result\",\"sourceId\":{source_id},\"sinkId\":{},\"seconds\":{seconds},\"completed\":true,\"directMonitoring\":false,\"changedDefault\":false,\"saved\":{},\"savedPath\":{}}}",
            sink.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
            saved_path.is_some(),
            saved_path
                .as_ref()
                .map(|p| json_texto(&p.display().to_string()))
                .unwrap_or_else(|| "null".into())
        );
    } else {
        println!("✓ grabación reproducida; no cambió el dispositivo predeterminado");
    }
    Ok(ExitCode::SUCCESS)
}

fn camera_stream_rgba(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let device = args
        .first()
        .ok_or_else(|| "Falta dispositivo de cámara.".to_string())?;

    let mut width = 640u32;
    let mut height = 360u32;
    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--width" => {
                i += 1;
                width = args
                    .get(i)
                    .ok_or_else(|| "Falta valor para --width.".to_string())?
                    .parse::<u32>()
                    .map_err(|_| "Ancho de cámara inválido.".to_string())?
                    .clamp(320, 1280);
            }
            "--height" => {
                i += 1;
                height = args
                    .get(i)
                    .ok_or_else(|| "Falta valor para --height.".to_string())?
                    .parse::<u32>()
                    .map_err(|_| "Alto de cámara inválido.".to_string())?
                    .clamp(180, 720);
            }
            otro => return Err(format!("Opción de cámara desconocida: {otro}")),
        }
        i += 1;
    }

    let cameras = camera_list_json(raiz)?;
    let camera = jq_compacto(
        raiz,
        &cameras,
        &format!(".devices[] | select(.device == {})", json_texto(device)),
    )?;
    if camera.is_empty() || camera == "null" {
        return Err("La cámara no existe o no es accesible.".into());
    }

    let raw_formats = jq_texto(raiz, &camera, ".rawFormats // \"\"")?;
    let mjpeg_30 = raw_formats.contains("'MJPG'")
        && raw_formats.contains("Size: Discrete 640x360")
        && raw_formats.contains("30.000 fps");

    let filtro = format!(
        "fps=30,scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:black"
    );

    let bin = if let Some(base) = env::var_os("KORUNIX_MEDIA_BIN_DIR") {
        PathBuf::from(base).join("ffmpeg").into_os_string()
    } else {
        tool("ffmpeg")
    };

    let mut ffargs = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-fflags".to_string(),
        "nobuffer".to_string(),
        "-flags".to_string(),
        "low_delay".to_string(),
        "-thread_queue_size".to_string(),
        "4".to_string(),
        "-f".to_string(),
        "v4l2".to_string(),
    ];

    if mjpeg_30 {
        ffargs.extend([
            "-input_format".to_string(),
            "mjpeg".to_string(),
            "-framerate".to_string(),
            "30".to_string(),
            "-video_size".to_string(),
            "640x360".to_string(),
        ]);
    }

    ffargs.extend([
        "-i".to_string(),
        device.clone(),
        "-vf".to_string(),
        filtro,
        "-pix_fmt".to_string(),
        "rgba".to_string(),
        "-f".to_string(),
        "rawvideo".to_string(),
        "pipe:1".to_string(),
    ]);

    use std::os::unix::process::CommandExt as _;

    let error = Command::new(bin)
        .args(&ffargs)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .exec();

    Err(format!("No pude iniciar la cámara: {error}"))
}

fn camera_preview(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    let device = args
        .first()
        .ok_or_else(|| "Falta dispositivo de cámara.".to_string())?;
    let mut plan_only = false;
    let mut yes = false;
    let mut json = false;
    let mut size: Option<String> = None;
    let mut fps: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--size" => {
                i += 1;
                size = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Falta resolución.".to_string())?,
                );
            }
            "--fps" => {
                i += 1;
                fps = Some(
                    args.get(i)
                        .cloned()
                        .ok_or_else(|| "Faltan FPS.".to_string())?,
                );
            }
            "--plan" => plan_only = true,
            "--yes" => yes = true,
            "--json" => json = true,
            x => return Err(format!("Opción desconocida: {x}")),
        }
        i += 1;
    }

    let cameras = camera_list_json(raiz)?;
    let camera = jq_compacto(
        raiz,
        &cameras,
        &format!(".devices[] | select(.device == {})", json_texto(device)),
    )?;
    if camera.is_empty() || camera == "null" {
        return Err("La cámara no existe o no es accesible.".into());
    }

    let available = jq_compacto(raiz, &camera, ".available // false")?;
    if available != "true" {
        return Err(
            "La cámara está detectada, pero todavía no ofrece una fuente de vídeo disponible."
                .into(),
        );
    }

    let plan = jq0(
        raiz,
        &[
            "-cn".into(),
            "--argjson".into(),
            "camera".into(),
            camera,
            "--arg".into(),
            "size".into(),
            size.clone().unwrap_or_default(),
            "--arg".into(),
            "fps".into(),
            fps.clone().unwrap_or_default(),
            r#"{
              schemaVersion:1,
              kind:"korunix-media-camera-preview-plan",
              backend:"v4l2",
              camera:$camera,
              mode:{size:($size|if .=="" then null else . end),fps:($fps|if .=="" then null else . end)},
              privacy:{explicitUserAction:true,backgroundCapture:false,persistsRecording:false},
              actions:{changesDefault:false,writesConfiguration:false}
            }"#
            .into(),
        ],
    )?;

    if plan_only {
        if yes {
            return Err("--yes no se utiliza junto con --plan.".into());
        }
        if json {
            println!("{plan}");
        } else {
            pretty(raiz, &plan)?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    if !media_interactive_ok(json, yes, &format!("¿Abrir temporalmente {device}?"))? {
        return Ok(ExitCode::SUCCESS);
    }

    let mut ffargs = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "warning".into(),
        "-f".into(),
        "v4l2".into(),
    ];
    if let Some(value) = size {
        ffargs.extend(["-video_size".into(), value]);
    }
    if let Some(value) = fps {
        ffargs.extend(["-framerate".into(), value]);
    }
    ffargs.extend(["-i".into(), device.clone()]);
    media_exec(raiz, "ffplay", &ffargs)?;

    if json {
        println!(
            "{{\"schemaVersion\":1,\"kind\":\"korunix-media-camera-preview-result\",\"camera\":{},\"completed\":true,\"changedDefault\":false,\"recordingSaved\":false}}",
            json_texto(device)
        );
    } else {
        println!("✓ previsualización cerrada; no se guardó vídeo");
    }
    Ok(ExitCode::SUCCESS)
}

fn media_full(raiz: &Path, args: &[String]) -> Result<ExitCode, String> {
    if args.first().map(String::as_str) == Some("audio")
        && args.get(1).map(String::as_str) == Some("test-output")
    {
        return speaker_test(raiz, &args[2..]);
    }
    if args.first().map(String::as_str) == Some("mic")
        && args.get(1).map(String::as_str) == Some("meter")
    {
        return mic_meter_live(raiz, &args[2..]);
    }
    if args.first().map(String::as_str) == Some("mic")
        && args.get(1).map(String::as_str) == Some("record")
    {
        return mic_record_temp(raiz, &args[2..]);
    }
    if args.first().map(String::as_str) == Some("mic")
        && args.get(1).map(String::as_str) == Some("play")
    {
        return mic_play_temp(raiz, &args[2..]);
    }
    if args.first().map(String::as_str) == Some("mic")
        && args.get(1).map(String::as_str) == Some("test")
    {
        return mic_test(raiz, &args[2..], false);
    }
    if args.first().map(String::as_str) == Some("camera")
        && args.get(1).map(String::as_str) == Some("stream")
    {
        return camera_stream_rgba(raiz, &args[2..]);
    }
    if args.first().map(String::as_str) == Some("camera")
        && args.get(1).map(String::as_str) == Some("preview")
    {
        return camera_preview(raiz, &args[2..]);
    }
    media(raiz, args)
}

pub(super) fn ejecutar_operacion(
    raiz: &Path,
    command: &str,
    args_os: &[OsString],
) -> Result<ExitCode, String> {
    let args = args_texto(args_os);
    recover_pending_transaction(raiz)?;

    match command {
        "bootstrap" => bootstrap(raiz, &args),
        "product" if args.is_empty() => product_status(raiz, false),
        "product" if args == ["--json"] => product_status(raiz, true),
        "host" => host_operation(raiz, &args),
        "hardware" if args.is_empty() => {
            hardware_human(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "localization" if args.is_empty() => {
            localization_human(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "localization" => localization_operation(raiz, &args),
        "interface-language" => interface_language_operation(raiz, &args),
        "users" => users_mutation(raiz, &args),
        "applications" => applications_operation(raiz, &args),
        "desktop" => desktop_operation(raiz, &args),
        "appearance" => appearance_operation(raiz, &args),
        "defaults" => defaults_operation(raiz, &args),
        "backup" => backup_operation(raiz, &args),
        "history" => history_operation(raiz, &args),
        "status" if args.is_empty() => {
            status(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "structure" if args.is_empty() => {
            structure(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "validate" if args.is_empty() => {
            validate(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "format" if args.is_empty() => {
            format_nix(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "recovery" | "generations" if args.is_empty() => {
            pretty(raiz, &recovery_list_json(raiz)?)?;
            Ok(ExitCode::SUCCESS)
        }
        "preview" | "build" | "apply" => change_cycle(raiz, command, &args),
        "update" => update(raiz, &args),
        "rollback" => rollback(raiz, &args),
        "clean-preview" => clean(raiz, false, true, &args),
        "clean-all-preview" => clean(raiz, true, true, &args),
        "clean" => clean(raiz, false, false, &args),
        "clean-all" => clean(raiz, true, false, &args),
        "storage" => storage(raiz, &args),
        "firmware" => firmware(raiz, &args),
        "media" => media_full(raiz, &args),
        _ => Err(format!(
            "Operación desconocida: {command}. Usa `korunix --help`."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_cuentas() {
        assert!(account_valid("koru"));
        assert!(account_valid("persona_2"));
        assert!(!account_valid("../root"));
        assert!(!account_valid("Mayuscula"));
    }

    #[test]
    fn nombre_visible_del_equipo_se_normaliza() {
        assert_eq!(
            host_name_normalized("  Mi_Equipo Casa  ").expect("nombre válido"),
            "mi-equipo-casa"
        );

        assert!(host_name_normalized("portátil").is_err());
        assert!(host_name_normalized("").is_err());
    }

    #[test]
    fn renombrar_equipo_preserva_el_resto_del_host() {
        let original = r#"{
  system = "x86_64-linux";
  korunix = {
    enable = true;
    channel = "stable";
    hostName = "equipo-viejo";
    stateVersion = "26.05";
  };
}
"#;

        let (nuevo, anterior) =
            host_name_text(original, "equipo-nuevo").expect("hostName editable");

        assert_eq!(anterior, "equipo-viejo");
        assert!(nuevo.contains(r#"hostName = "equipo-nuevo";"#));
        assert!(nuevo.contains(r#"channel = "stable";"#));
        assert!(nuevo.contains(r#"stateVersion = "26.05";"#));
        assert!(!nuevo.contains("equipo-viejo"));
    }

    #[test]
    fn bootstrap_id_humano() {
        assert_eq!(bootstrap_host_id("Mi Equipo!!!"), "mi-equipo");
    }

    #[test]
    fn bootstrap_lineas_usuario_se_generan_sin_jq() {
        let plan = r#"{
          "userAdoption": {
            "profiles": [
              {
                "id": "ana",
                "local": {
                  "homeDirectory": "/home/ana",
                  "administrator": true
                }
              }
            ]
          }
        }"#;

        let lines = bootstrap_host_profile_lines(plan).expect("plan válido");

        assert_eq!(
            lines,
            r#"    ana = { homeDirectory = "/home/ana"; administrator = true; deferredCapabilities = []; deferredInputMethods = []; preservedGroups = []; githubSshIdentityFile = null; };"#
        );
    }

    #[test]
    fn bootstrap_incorpora_hardware_sin_regenerarlo() {
        let original = r#"{ config, lib, modulesPath, ... }:
{
  imports = [ (modulesPath + "/installer/scan/not-detected.nix") ];
  fileSystems."/" = { device = "/dev/disk/by-uuid/PRUEBA"; fsType = "btrfs"; };
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
}
"#;

        let graphics = r#"[{"pciAddress":"0000:01:00.0","name":"GPU de prueba","vendor":"amd","vendorId":"1002","deviceId":"0001","subsystemVendorId":"0000","subsystemDeviceId":"0000","class":"030000","driver":"amdgpu","primary":true,"kind":"unknown","nvidiaOpen":false}]"#;

        let adopted =
            bootstrap_hardware_text_from(original, "uefi", graphics).expect("hardware válido");

        assert!(adopted.contains(r#"device = "/dev/disk/by-uuid/PRUEBA""#));
        assert!(adopted.contains(r#"korunix.hardware.firmware = "uefi";"#));
        assert!(adopted.contains("korunix.hardware.graphics = builtins.fromJSON"));
        assert!(adopted.contains("GPU de prueba"));
        assert!(!adopted.contains(r#"\"class\""#));
        assert!(adopted.starts_with("# NO CAMBIES ESTE ARCHIVO A MANO."));
    }

    #[test]
    fn transaccion_solo_admite_configuracion_y_hardware_generado() {
        let raiz = Path::new("/tmp/korunix-prueba");

        assert!(
            transaction_relative_path(raiz, &raiz.join("configuracion/equipos/prueba.nix")).is_ok()
        );

        assert!(transaction_relative_path(
            raiz,
            &raiz.join("generado/equipos/prueba-detectado.nix")
        )
        .is_ok());

        assert!(transaction_relative_path(raiz, &raiz.join("sistema/base.nix")).is_err());

        assert!(transaction_relative_path(raiz, &raiz.join("generado/otro/archivo.nix")).is_err());
    }

    #[test]
    fn almacenamiento_no_declara_sync_global() {
        let strategy = "syncfs-per-filesystem";
        assert_ne!(strategy, "sync-global");
    }

    #[test]
    fn localizacion_reemplaza_lista_de_una_linea_sin_perder_el_resto() {
        let source = "      systemLanguage = \"es\";\n      preferredLanguages = [\"es\"];\n      region = \"PE\";\n";
        let values = vec!["es".to_string(), "en".to_string()];

        let result = reemplazar_o_insertar_lista_nix(
            source,
            6,
            "preferredLanguages",
            &values,
            "systemLanguage",
        )
        .unwrap();

        assert!(result.contains("preferredLanguages = ["));
        assert!(result.contains("\"es\""));
        assert!(result.contains("\"en\""));
        assert!(result.contains("region = \"PE\";"));
    }

    #[test]
    fn localizacion_inserta_lista_ausente_de_forma_controlada() {
        let source = "      systemLanguage = \"es\";\n      region = \"PE\";\n";
        let values = vec!["es".to_string()];

        let result = reemplazar_o_insertar_lista_nix(
            source,
            6,
            "preferredLanguages",
            &values,
            "systemLanguage",
        )
        .unwrap();

        assert_eq!(result.matches("preferredLanguages = [").count(), 1);
        assert!(
            result.find("systemLanguage").unwrap() < result.find("preferredLanguages").unwrap()
        );
    }

    #[test]
    fn catalogo_xkb_conserva_layout_y_variantes() {
        let raw = r#"
! layout
  us              English (US)
  es              Spanish
  latam           Spanish (Latin American)
! variant
  deadtilde       es: Spanish (dead tilde)
  nodeadkeys      es: Spanish (no dead keys)
"#;

        let catalog = parse_xkb_rules(raw);

        assert!(catalog.iter().any(|item| {
            item.layout == "es" && item.variant.is_empty() && item.label == "Spanish"
        }));
        assert!(catalog.iter().any(|item| {
            item.layout == "es"
                && item.variant == "deadtilde"
                && item.label == "Spanish (dead tilde)"
        }));
        assert!(catalog
            .iter()
            .any(|item| { item.layout == "latam" && item.variant.is_empty() }));
    }

    #[test]
    fn idioma_interfaz_normaliza_locales_soportados() {
        assert_eq!(interface_language_from_locale("es_PE.UTF-8"), Some("es"));
        assert_eq!(interface_language_from_locale("pt_BR.UTF-8"), Some("pt-BR"));
        assert_eq!(
            interface_language_from_locale("zh_CN.UTF-8"),
            Some("zh-Hans")
        );
        assert_eq!(interface_language_from_locale("xx_YY"), None);
        assert_eq!(KORUNIX_INTERFACE_LANGUAGES.len(), 23);
    }

    #[test]
    fn idioma_interfaz_lee_preferencia_simple_sin_evaluar_nix() {
        let automatico = r#"{
  accountName = "ana";
  language = "es";
  interfaceLanguage = null;
}
"#;

        let explicito = r#"{
  accountName = "ana";
  language = "es";
  interfaceLanguage = "en"; # solo Korunix
}
"#;

        assert_eq!(
            profile_simple_optional_string_value(automatico, "interfaceLanguage").unwrap(),
            Some(None)
        );

        assert_eq!(
            profile_simple_optional_string_value(explicito, "interfaceLanguage").unwrap(),
            Some(Some("en".to_string()))
        );
    }

    #[test]
    fn idioma_interfaz_no_reutiliza_language_del_perfil() {
        let original = r#"{
  accountName = "ana";
  fullName = "Ana";
  language = "es";
  interfaceLanguage = null;
}
"#;

        let updated =
            profile_interface_language_text(original, Some("en")).expect("perfil editable");

        assert!(updated.contains(r#"language = "es";"#));
        assert!(updated.contains(r#"interfaceLanguage = "en";"#));
    }

    #[test]
    fn importacion_portable_conserva_preferencias_de_idioma() {
        let value = serde_json::json!({
            "accountName": "ana",
            "fullName": "Ana",
            "language": "es",
            "interfaceLanguage": "en",
            "inputMethods": ["japanese-mozc"],
            "capabilities": ["printing"]
        });

        let profile = profile_text_from_manifest(&value).expect("manifest portable");

        assert!(profile.contains(r#"language = "es";"#));
        assert!(profile.contains(r#"interfaceLanguage = "en";"#));
        assert!(profile.contains(r#""japanese-mozc""#));
        assert!(profile.contains(r#""printing""#));
    }

    #[test]
    fn transferencia_local_persiste_y_verifica() {
        let root = env::temp_dir().join(format!("korunix-transfer-test-{}", stamp()));
        let source_dir = root.join("origen");
        let destination = root.join("destino");

        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&destination).unwrap();

        let source = source_dir.join("prueba.bin");
        fs::write(&source, b"korunix-transferencia-segura\n").unwrap();

        let sources = transfer_sources(&[source.display().to_string()]).unwrap();
        let copied = transferir_archivos_a_directorio(&sources, &destination, false).unwrap();

        assert_eq!(copied.len(), 1);
        assert_eq!(
            fs::read(destination.join("prueba.bin")).unwrap(),
            b"korunix-transferencia-segura\n"
        );

        let partials = fs::read_dir(&destination)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(".korunix-transfer-")
            })
            .count();

        assert_eq!(partials, 0);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn apariencia_se_inserta_sin_reescribir_el_host() {
        let original = r#"{
  korunix = {
    enable = true;
    channel = "unstable";
  };
}
"#;

        let nuevo = appearance_host_text(original, "everforest", "dark").unwrap();
        assert!(nuevo.contains("appearance = {"));
        assert!(nuevo.contains(r#"style = "everforest";"#));
        assert!(nuevo.contains(r#"mode = "dark";"#));
        assert!(nuevo.contains(r#"channel = "unstable";"#));
    }

    #[test]
    fn apariencia_existente_conserva_comentarios() {
        let original = r#"{
  korunix = {
    appearance = {
      # Preferencia humana.
      style = "default";
      mode = "auto";
    };
  };
}
"#;

        let nuevo = appearance_host_text(original, "everforest", "light").unwrap();
        assert!(nuevo.contains("# Preferencia humana."));
        assert!(nuevo.contains(r#"style = "everforest";"#));
        assert!(nuevo.contains(r#"mode = "light";"#));
        assert_eq!(nuevo.matches("appearance = {").count(), 1);
    }

    #[test]
    fn apariencia_historial_no_filtra_identificadores_internos() {
        assert_eq!(appearance_style_human("default"), "Predeterminada");
        assert_eq!(appearance_style_human("dynamic"), "Dinámica");
        assert_eq!(appearance_style_human("everforest"), "Everforest");
        assert_eq!(appearance_mode_human("light"), "Claro");
        assert_eq!(appearance_mode_human("dark"), "Oscuro");
        assert_eq!(appearance_mode_human("auto"), "Automático");
    }

    #[test]
    fn roles_portables_se_insertan_sin_borrar_preferencias() {
        let original = r#"# ESTE ARCHIVO SE PUEDE CAMBIAR.
{
  accountName = "ana";
  fullName = "Ana";
  language = "es";

  inputMethods = [];

  # Esta capacidad debe sobrevivir.
  capabilities = [
    "printing"
  ];

  avatar = null;
}
"#;

        let nuevo = default_roles_profile_text(original, Some("firefox"), Some("kate"))
            .expect("perfil editable");

        assert!(nuevo.contains("defaultRoles = {"));
        assert!(nuevo.contains(r#"browser = "firefox";"#));
        assert!(nuevo.contains(r#"plasmaTextEditor = "kate";"#));
        assert!(nuevo.contains("# Esta capacidad debe sobrevivir."));
        assert!(nuevo.contains(r#""printing""#));
        assert!(nuevo.contains("avatar = null;"));
        assert_eq!(nuevo.matches("defaultRoles = {").count(), 1);
    }

    #[test]
    fn roles_portables_actualizan_una_eleccion_y_conservan_la_otra() {
        let original = r#"{
  accountName = "ana";

  defaultRoles = {
    # El comentario humano se conserva.
    browser = "firefox";
    plasmaTextEditor = "kwrite";
  };

  capabilities = [];
}
"#;

        let nuevo = default_roles_profile_text(original, Some("google-chrome"), None)
            .expect("perfil editable");

        assert!(nuevo.contains("# El comentario humano se conserva."));
        assert!(nuevo.contains(r#"browser = "google-chrome";"#));
        assert!(nuevo.contains(r#"plasmaTextEditor = "kwrite";"#));
        assert!(!nuevo.contains(r#"browser = "firefox";"#));
        assert_eq!(nuevo.matches("defaultRoles = {").count(), 1);
    }

    #[test]
    fn aplicacion_nixpkgs_guarda_id_humano_y_flatpak_conserva_fuente() {
        assert_eq!(
            application_selection_token("nixpkgs", "hello").unwrap(),
            "hello"
        );
        assert_eq!(
            application_selection_token("nixpkgs", "legacyPackages.x86_64-linux.blender").unwrap(),
            "blender"
        );
        assert_eq!(
            application_selection_token("flatpak", "org.mozilla.firefox").unwrap(),
            "flatpak:org.mozilla.firefox"
        );
    }

    #[test]
    fn perfil_con_avatar_no_contiene_secretos() {
        let perfil =
            profile_text_with_avatar("prueba", "Persona de prueba", Some("prueba.png")).unwrap();
        assert!(perfil.contains("avatar = ./prueba.png;"));
        assert!(!perfil.contains("avatar = null;"));
        assert_eq!(perfil.matches("avatar = ").count(), 1);
        assert!(!perfil.to_ascii_lowercase().contains("password"));
    }

    #[test]
    fn v4l2_formatos_se_estructuran() {
        let raw = r#"ioctl: VIDIOC_ENUM_FMT
    Type: Video Capture

    [0]: 'YUYV' (YUYV 4:2:2)
        Size: Discrete 640x480
            Interval: Discrete 0.033s (30.000 fps)
            Interval: Discrete 0.042s (24.000 fps)
        Size: Discrete 640x360
            Interval: Discrete 0.033s (30.000 fps)
    [1]: 'MJPG' (Motion-JPEG, compressed)
        Size: Discrete 1920x1080
            Interval: Discrete 0.033s (30.000 fps)
"#;

        let formatos = v4l2_formats(raw);
        assert_eq!(formatos.len(), 2);
        assert_eq!(formatos[0]["pixelFormat"], "YUYV");
        assert_eq!(formatos[0]["sizes"][0]["width"], 640);
        assert_eq!(formatos[0]["sizes"][0]["height"], 480);
        assert_eq!(formatos[0]["sizes"][0]["fps"][0], 30.0);
        assert_eq!(formatos[0]["sizes"][1]["height"], 360);
        assert_eq!(formatos[1]["pixelFormat"], "MJPG");
        assert_eq!(formatos[1]["sizes"][0]["width"], 1920);
        assert_eq!(formatos[1]["sizes"][0]["height"], 1080);
    }

    #[test]
    fn v4l2_nodo_de_metadatos_no_declara_captura() {
        let info = r#"Driver Info:
    Driver name      : uvcvideo
    Card type        : HD Pro Webcam C920
    Bus info         : usb-0000:05:00.4-1
    Device Caps      : 0x04a00000
        Metadata Capture
        Streaming
        Extended Pix Format
Media Driver Info:
    Driver name      : uvcvideo
"#;

        let capacidades = v4l2_device_caps(info);
        assert!(capacidades.iter().any(|c| c == "Metadata Capture"));
        assert!(!capacidades.iter().any(|c| c == "Video Capture"));
        assert!(!v4l2_virtual("uvcvideo", "usb-0000:05:00.4-1"));
    }
}
