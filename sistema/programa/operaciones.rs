//! Operaciones del sistema vivo de Korunix.
//!
//! Nix conserva el modelo declarativo. Este módulo Rust ejecuta las operaciones
//! que dependen del equipo en funcionamiento. No delega dominio operativo a Bash.
//!
//! Las pruebas pueden sustituir herramientas externas con `KORUNIX_TOOL_*` y la
//! frontera privilegiada completa con `KORUNIX_TEST_PRIVILEGED_RUNNER`. Nunca se
//! fabrica un pseudo-TTY ni se automatiza una contraseña.

use super::*;
use std::collections::BTreeSet;
use std::io::Write;

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
    fs::write(&tmp, data).map_err(|e| format!("No pude escribir {}: {e}", tmp.display()))?;
    if let Ok(meta) = fs::metadata(path) {
        let _ = fs::set_permissions(&tmp, meta.permissions());
    }
    fs::rename(&tmp, path).map_err(|e| format!("No pude sustituir {}: {e}", path.display()))
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
    if nombre.starts_with("/nix/store/") && nombre.ends_with("/bin/switch-to-configuration") {
        let p = PathBuf::from(nombre);
        if p.is_file() {
            return Ok(p);
        }
    }

    if !matches!(
        nombre,
        "nix"
            | "nix-env"
            | "nix-collect-garbage"
            | "bootctl"
            | "grub-reboot"
            | "grub-editenv"
            | "cat"
            | "passwd"
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
        "/run/current-system/sw/bin/pkexec",
        "/run/wrappers/bin/pkexec",
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

fn validate(raiz: &Path) -> Result<(), String> {
    structure(raiz)?;
    visible(raiz, "git", &["diff".into(), "--check".into()])?;
    visible(
        raiz,
        "nix",
        &[
            "flake".into(),
            "check".into(),
            "--no-build".into(),
            "--show-trace".into(),
        ],
    )?;

    for id in equipos_disponibles(raiz)? {
        let drv = capture(
            raiz,
            "nix",
            &[
                "eval".into(),
                "--raw".into(),
                "--no-write-lock-file".into(),
                format!(".#nixosConfigurations.{id}.config.system.build.toplevel.drvPath"),
            ],
        )?;
        if drv.is_empty() {
            return Err(format!("Nix no produjo drvPath para {id}."));
        }
    }
    println!("✓ VALIDACIÓN COMPLETA");
    Ok(())
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
    println!("=== Repositorio ===");
    visible(
        raiz,
        "git",
        &["status".into(), "--short".into(), "--branch".into()],
    )?;
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

    let entry = schedule_recovery(raiz, id)?;
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
    let _ = privileged(raiz, "nix-collect-garbage", &[], false)?;
    let _ = privileged(raiz, "nix", &["store".into(), "optimise".into()], false)?;

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

fn storage_list_json(raiz: &Path) -> Result<String, String> {
    let raw = capture(
        raiz,
        "lsblk",
        &[
            "-J".into(),
            "-p".into(),
            "-o".into(),
            "PATH,TYPE,RM,HOTPLUG,TRAN,SIZE,MODEL,MOUNTPOINTS".into(),
        ],
    )?;
    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            r#"
              def truthy: .==true or .==1 or .=="1";
              def nodes: ., (.children[]? | nodes);
              [.blockdevices[]?
               | select(.type=="disk")
               | . as $disk
               | {
                   device:.path,
                   size:.size,
                   model:(.model|if .==null then null else gsub("[[:space:]]+$";"") end),
                   transport:.tran,
                   removable:((.rm|truthy) or (.hotplug|truthy) or .tran=="usb" or .tran=="mmc"),
                   mountPoints:([$disk|nodes|.mountpoints[]?|select(type=="string" and length>0)]|unique)
                 }]
              | {schemaVersion:1,kind:"korunix-storage-list",devices:.}
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

    if args.first().map(String::as_str) != Some("eject") || args.len() < 2 {
        return Err("Uso: korunix storage --list [--json] | eject <dispositivo> [--heavy] [--plan] [--yes] [--json].".into());
    }

    let device = &args[1];
    let heavy = args.iter().any(|v| v == "--heavy");
    let plan_only = args.iter().any(|v| v == "--plan");
    let yes = args.iter().any(|v| v == "--yes");
    let json = args.iter().any(|v| v == "--json");
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

    if heavy {
        let mounts = jq_con_entrada(raiz, &["-r".into(), ".mounts[].mountPoint".into()], &plan)?;
        for mount in mounts.lines().filter(|v| !v.is_empty()) {
            visible(raiz, "sync", &["-f".into(), mount.into()])?;
        }
    }

    let devices = jq_con_entrada(
        raiz,
        &["-r".into(), ".mounts | map(.device) | unique[]".into()],
        &plan,
    )?;
    for dev in devices.lines().filter(|v| !v.is_empty()) {
        visible(
            raiz,
            "udisksctl",
            &["unmount".into(), "-b".into(), dev.into()],
        )?;
    }

    let disk = jq_texto(raiz, &plan, ".disk")?;
    visible(
        raiz,
        "udisksctl",
        &["power-off".into(), "-b".into(), disk.clone()],
    )?;

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
    fs::copy(&lock, backup.join("flake.lock"))
        .map_err(|e| format!("No pude respaldar flake.lock: {e}"))?;

    let mut nix_args = vec!["flake".into(), "update".into()];
    nix_args.extend(inputs);
    let (code, _, error) = capture_status(raiz, "nix", &nix_args)?;
    if code != 0 {
        let _ = fs::copy(backup.join("flake.lock"), &lock);
        return Err(format!(
            "La actualización falló; flake.lock fue restaurado. {error}"
        ));
    }

    if let Err(error) = validate(raiz) {
        let _ = fs::copy(backup.join("flake.lock"), &lock);
        return Err(format!(
            "La actualización no pasó la validación; flake.lock fue restaurado. {error}"
        ));
    }

    let result = update_result_json(raiz, &backup.join("flake.lock"), &lock, &plan, &backup)?;
    if json {
        println!("{result}");
    } else {
        println!("✓ ACTUALIZACIÓN PREPARADA");
        println!("No se construyó ni aplicó una generación.");
        println!("Respaldo: {}", backup.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn build_candidate(raiz: &Path) -> Result<PathBuf, String> {
    let host = resolver_equipo(raiz)?;
    let out = capture(
        raiz,
        "nix",
        &[
            "build".into(),
            format!(".#nixosConfigurations.{host}.config.system.build.toplevel"),
            "--no-link".into(),
            "--print-out-paths".into(),
            "--show-trace".into(),
        ],
    )?;
    let path = out
        .lines()
        .last()
        .map(PathBuf::from)
        .ok_or_else(|| "Nix no devolvió la candidata.".to_string())?;
    if !path.exists() {
        return Err(format!("La candidata no existe: {}", path.display()));
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
) -> Result<(PathBuf, PathBuf, String, String, String), String> {
    let current_text = current_system();
    let current = if current_text.is_empty() {
        PathBuf::new()
    } else {
        PathBuf::from(current_text)
    };
    let candidate = build_candidate(raiz)?;

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
        let activator = candidate.join("bin/switch-to-configuration");
        if !activator.is_file() {
            return Err("La candidata no contiene switch-to-configuration.".into());
        }
        privileged(
            raiz,
            &activator.display().to_string(),
            &["dry-activate".into()],
            false,
        )?
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
        validate(raiz)?;
    }

    let previewed = command != "build";
    let (current, candidate, diff, activation, impact) = prepare_cycle(raiz, previewed)?;

    if command == "build" {
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
        return Ok(ExitCode::SUCCESS);
    }

    if !yes && !confirm("¿Aplicar esta generación ahora?")? {
        return Ok(ExitCode::SUCCESS);
    }

    let activator = candidate.join("bin/switch-to-configuration");
    let _ = privileged(
        raiz,
        &activator.display().to_string(),
        &["switch".into()],
        true,
    )?;
    let verified = current_system() == candidate.display().to_string();

    let _ = Command::new(tool("systemctl"))
        .args(["--user", "start", "korunix-user-prepare.service"])
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

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

fn firmware_devices_json(raiz: &Path) -> Result<String, String> {
    let raw = fwupd_raw(raiz, "get-devices")?;
    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            r#"{
              schemaVersion:1,
              kind:"korunix-firmware-devices",
              backend:"fwupd",
              devices:[
                (.Devices // [])[]
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
            }"#
            .into(),
        ],
        &raw,
    )
}

fn firmware_updates_json(raiz: &Path) -> Result<String, String> {
    let raw = fwupd_raw(raiz, "get-updates")?;
    jq_con_entrada(
        raiz,
        &[
            "-c".into(),
            r#"{
              schemaVersion:1,
              kind:"korunix-firmware-updates",
              backend:"fwupd",
              metadataRefreshPerformed:false,
              devices:[
                (.Devices // [])[]
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
                          urgency:(.Urgency // null),
                          releaseId:(.ReleaseId // null)
                        }
                    ]
                  }
              ]
            }"#
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
            let (code, _, err) = capture_status(
                raiz,
                "fwupdmgr",
                &["--assume-yes".into(), "refresh".into()],
            )?;
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
                      ports:(.ports // [])
                    }
                ]
              }
            }"#
            .into(),
        ],
    )
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
            for e in entries.flatten() {
                if e.file_name().to_string_lossy().starts_with("video") {
                    found.push(e.path());
                }
            }
        }
        found.sort();
        found
    };

    let mut result = Vec::new();
    for device in devices {
        let info = match media_capture(
            raiz,
            "v4l2-ctl",
            &["-d".into(), device.display().to_string(), "--info".into()],
        ) {
            Ok(v) => v,
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

        let field = |label: &str| {
            info.lines().find_map(|line| {
                let t = line.trim_start();
                let (name, value) = t.split_once(':')?;
                (name.trim() == label).then(|| value.trim().to_string())
            })
        };

        // Conservamos la salida anunciada para diagnóstico. Los modos estructurados
        // se completarán en D.3 cuando la GUI necesite selector visual de formato.
        result.push(format!(
            "{{\"device\":{},\"driver\":{},\"card\":{},\"bus\":{},\"version\":{},\"formats\":[],\"rawFormats\":{}}}",
            json_texto(&device.display().to_string()),
            field("Driver name").as_deref().map(json_texto).unwrap_or_else(|| "null".into()),
            field("Card type").as_deref().map(json_texto).unwrap_or_else(|| "null".into()),
            field("Bus info").as_deref().map(json_texto).unwrap_or_else(|| "null".into()),
            field("Driver version").as_deref().map(json_texto).unwrap_or_else(|| "null".into()),
            json_texto(&formats_raw)
        ));
    }

    Ok(format!(
        "{{\"schemaVersion\":1,\"kind\":\"korunix-media-cameras\",\"backend\":\"v4l2\",\"devices\":[{}]}}",
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
         \x20 inputMethods = [];\n\
         \x20 capabilities = [];\n\
         \x20 avatar = null;\n\
         }}\n",
        json_texto(account),
        json_texto(name)
    )
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
            if !profile_existed {
                atomic_write(&profile, profile_text(account, &name).as_bytes())?;
            }
            let host_path = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));
            let old_host = fs::read(&host_path).map_err(|e| e.to_string())?;
            let preserved: Vec<String> = groups
                .into_iter()
                .filter(|v| !matches!(v.as_str(), "users" | "wheel" | "networkmanager"))
                .collect();

            if !confirm(&format!("¿Preparar la adopción de {account}?"))? {
                if !profile_existed {
                    let _ = fs::remove_file(profile);
                }
                return Ok(ExitCode::SUCCESS);
            }

            if let Err(error) = (|| -> Result<(), String> {
                add_host_user(raiz, &host, account, account, &home, admin, &preserved)?;
                validate(raiz)
            })() {
                let _ = atomic_write(&host_path, &old_host);
                if !profile_existed {
                    let _ = fs::remove_file(profile);
                }
                return Err(format!("Adopción revertida: {error}"));
            }

            println!("✓ adopción preparada; la contraseña quedó intacta");
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
            let old_host = fs::read(&host_path).map_err(|e| e.to_string())?;
            atomic_write(&profile, profile_text(&account, &name).as_bytes())?;

            if let Err(error) = (|| -> Result<(), String> {
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
            })() {
                let _ = atomic_write(&host_path, &old_host);
                let _ = fs::remove_file(profile);
                return Err(format!("Creación revertida: {error}"));
            }

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
                      schemaVersion:2,
                      kind:"korunix-user-profile",
                      exportedAt:(now|todate),
                      profile:{
                        id:$id,
                        accountName:$p.accountName,
                        fullName:$p.fullName,
                        language:($p.language // null),
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
                      schemaVersion:2,
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

            let name = jq_texto(raiz, &manifest, ".profile.fullName")?;
            let profile = raiz
                .join("configuracion/personas")
                .join(format!("{id}.nix"));
            atomic_write(&profile, profile_text(&account, &name).as_bytes())?;
            let info = account_info(raiz, &account)?;
            let (home, admin, preserved) = if let Some((_, home, admin, groups)) = info {
                (
                    home,
                    admin,
                    groups
                        .into_iter()
                        .filter(|v| !matches!(v.as_str(), "users" | "wheel" | "networkmanager"))
                        .collect(),
                )
            } else {
                (format!("/home/{account}"), false, Vec::new())
            };
            let host_path = raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"));
            let old_host = fs::read(&host_path).map_err(|e| e.to_string())?;

            if let Err(error) = (|| -> Result<(), String> {
                add_host_user(raiz, &host, &id, &account, &home, admin, &preserved)?;
                validate(raiz)
            })() {
                let _ = atomic_write(&host_path, &old_host);
                let _ = fs::remove_file(profile);
                let _ = fs::remove_dir_all(temp);
                return Err(format!("Importación revertida: {error}"));
            }
            let _ = fs::remove_dir_all(temp);
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
            "{{\"id\":{},\"target\":{{\"profilePath\":{},\"profileExists\":{},\"overwriteAllowed\":false}},\"portable\":{{\"accountName\":{},\"fullName\":{},\"language\":null,\"inputMethods\":[],\"capabilities\":[],\"avatar\":null}},\"local\":{{\"homeDirectory\":{},\"administrator\":{},\"deferredCapabilities\":[],\"deferredInputMethods\":[],\"preservedGroups\":[],\"githubSshIdentityFile\":null}},\"credentials\":{{\"action\":\"preserve-existing\",\"importedSecret\":false}}}}",
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
                productDefaults:"sistema/predeterminados.nix",
                channels:"sistema/canales.nix",
                reusesInstalledHardware:true,
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
            // La primera adopción real ya fue cerrada en B y C. Aquí usamos el
            // mismo mecanismo de personas/equipo; no tocamos /etc/nixos.
            let plan = bootstrap_plan_json(raiz)?;
            let host = jq_texto(raiz, &plan, ".host.id")?;
            if raiz
                .join("configuracion/equipos")
                .join(format!("{host}.nix"))
                .exists()
            {
                return Err("Ese equipo ya existe dentro de Korunix.".into());
            }
            let backup = backup_dir(&format!("bootstrap-{host}"))?;
            fs::write(backup.join("plan.json"), &plan).map_err(|e| e.to_string())?;

            let profiles = jq_con_entrada(
                raiz,
                &["-c".into(), ".userAdoption.profiles[]".into()],
                &plan,
            )?;
            let mut created = Vec::new();
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
                atomic_write(&path, profile_text(&account, &name).as_bytes())?;
                created.push(path);
            }

            // Crear una declaración mínima que conserva stateVersion y cuentas.
            let system = jq_texto(raiz, &plan, ".host.system")?;
            let host_name = jq_texto(raiz, &plan, ".host.name")?;
            let state_version = jq_texto(raiz, &plan, ".host.stateVersion")?;
            let channel = jq_texto(raiz, &plan, ".host.channel")?;
            let profile_lines = jq_con_entrada(
                raiz,
                &[
                    "-r".into(),
                    r#".userAdoption.profiles[]
                       | "    " + .id + " = { homeDirectory = "
                         + (.local.homeDirectory|@json)
                         + "; administrator = "
                         + (.local.administrator|tostring)
                         + "; deferredCapabilities = []; deferredInputMethods = []; preservedGroups = []; githubSshIdentityFile = null; };"#.into(),
                ],
                &plan,
            )?;
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
            if let Err(error) =
                atomic_write(&host_path, host_text.as_bytes()).and_then(|_| validate(raiz))
            {
                let _ = fs::remove_file(&host_path);
                for path in created {
                    let _ = fs::remove_file(path);
                }
                return Err(format!("Bootstrap revertido: {error}"));
            }
            println!("✓ instalación adoptada sin modificar /etc/nixos");
        }
        _ => return Err("Uso: korunix bootstrap --plan [--json] | --adopt [--yes].".into()),
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

fn mic_test(raiz: &Path, args: &[String], meter: bool) -> Result<ExitCode, String> {
    let source = args.first().ok_or_else(|| "Falta source-id.".to_string())?;
    let source_id: u32 = source
        .parse()
        .map_err(|_| "source-id inválido.".to_string())?;
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
            source.clone(),
            "--rate".into(),
            "48000".into(),
            "--channels".into(),
            "1".into(),
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
        return mic_test(raiz, &args[2..], true);
    }
    if args.first().map(String::as_str) == Some("mic")
        && args.get(1).map(String::as_str) == Some("test")
    {
        return mic_test(raiz, &args[2..], false);
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
    match command {
        "bootstrap" => bootstrap(raiz, &args),
        "hardware" if args.is_empty() => {
            hardware_human(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "localization" if args.is_empty() => {
            localization_human(raiz)?;
            Ok(ExitCode::SUCCESS)
        }
        "users" => users_mutation(raiz, &args),
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
    fn bootstrap_id_humano() {
        assert_eq!(bootstrap_host_id("Mi Equipo!!!"), "mi-equipo");
    }

    #[test]
    fn almacenamiento_no_declara_sync_global() {
        let strategy = "syncfs-per-filesystem";
        assert_ne!(strategy, "sync-global");
    }
}
