use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnidadLocal {
    pub nombre: String,
    pub detalle: String,
}

#[derive(Debug, Deserialize)]
struct SalidaLsblk {
    #[serde(default)]
    blockdevices: Vec<Dispositivo>,
}

#[derive(Debug, Deserialize)]
struct Dispositivo {
    #[serde(default)]
    size: Option<Value>,
    #[serde(rename = "type", default)]
    tipo: Option<String>,
    #[serde(default)]
    fstype: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    tran: Option<String>,
    #[serde(default)]
    mountpoints: Option<Vec<Option<String>>>,
    #[serde(default)]
    children: Vec<Dispositivo>,
}

fn bytes(valor: &Option<Value>) -> Option<u64> {
    match valor.as_ref()? {
        Value::Number(numero) => numero.as_u64(),
        Value::String(texto) => texto.parse().ok(),
        _ => None,
    }
}

fn tiene_montaje(dispositivo: &Dispositivo, buscado: Option<&str>) -> bool {
    let aqui = dispositivo
        .mountpoints
        .as_deref()
        .unwrap_or_default()
        .iter()
        .flatten()
        .any(|montaje| match buscado {
            Some(buscado) => montaje == buscado,
            None => !montaje.is_empty(),
        });

    aqui || dispositivo
        .children
        .iter()
        .any(|hijo| tiene_montaje(hijo, buscado))
}

fn particion_principal(dispositivo: &Dispositivo) -> Option<&Dispositivo> {
    let mut candidatas: Vec<&Dispositivo> = dispositivo
        .children
        .iter()
        .filter(|hijo| {
            hijo.fstype
                .as_deref()
                .is_some_and(|valor| !valor.is_empty())
        })
        .collect();

    candidatas.sort_by_key(|candidata| bytes(&candidata.size).unwrap_or(0));
    candidatas.pop()
}

fn capacidad_humana(bytes: u64) -> String {
    let gb = (bytes.saturating_add(500_000_000)) / 1_000_000_000;
    format!("{gb} GB")
}

fn conexion_humana(valor: Option<&str>) -> Option<String> {
    let valor = valor?.trim();

    if valor.is_empty() {
        return None;
    }

    Some(match valor.to_ascii_lowercase().as_str() {
        "sata" | "ata" => "SATA".to_string(),
        "usb" => "USB".to_string(),
        "nvme" => "NVMe".to_string(),
        otro => otro.to_uppercase(),
    })
}

fn sistema_humano(valor: Option<&str>) -> Option<String> {
    let valor = valor?.trim();

    if valor.is_empty() {
        return None;
    }

    Some(match valor.to_ascii_lowercase().as_str() {
        "ntfs" | "ntfs3" => "NTFS".to_string(),
        "vfat" | "fat" | "fat32" => "FAT".to_string(),
        "exfat" => "exFAT".to_string(),
        otro => otro.to_string(),
    })
}

fn leer_json(datos: &[u8]) -> Result<Vec<UnidadLocal>, String> {
    let salida: SalidaLsblk = serde_json::from_slice(datos).map_err(|error| {
        format!("No pude entender el estado local de los discos.\nDetalle: {error}")
    })?;

    let mut unidades = Vec::new();

    for disco in salida.blockdevices {
        if disco.tipo.as_deref() != Some("disk") || tiene_montaje(&disco, Some("/")) {
            continue;
        }

        let modelo = disco.model.as_deref().unwrap_or("").trim().to_string();
        let capacidad = bytes(&disco.size)
            .map(capacidad_humana)
            .unwrap_or_else(|| "capacidad desconocida".to_string());

        let nombre = if modelo.is_empty() {
            format!("Disco · {capacidad}")
        } else {
            format!("{modelo} · {capacidad}")
        };

        let particion = particion_principal(&disco);
        let mut detalles = Vec::new();

        if let Some(conexion) = conexion_humana(disco.tran.as_deref()) {
            detalles.push(conexion);
        }

        if let Some(sistema) = sistema_humano(particion.and_then(|parte| parte.fstype.as_deref())) {
            detalles.push(sistema);
        }

        if let Some(etiqueta) = particion
            .and_then(|parte| parte.label.as_deref())
            .map(str::trim)
            .filter(|etiqueta| !etiqueta.is_empty())
        {
            detalles.push(format!("Etiqueta: {etiqueta}"));
        }

        unidades.push(UnidadLocal {
            nombre,
            detalle: detalles.join(" · "),
        });
    }

    unidades.sort_by(|izquierda, derecha| izquierda.nombre.cmp(&derecha.nombre));
    Ok(unidades)
}

pub fn leer() -> Result<Vec<UnidadLocal>, String> {
    let programa = env::var_os("KORUNIX_LSBLK_BIN").unwrap_or_else(|| {
        if Path::new("/run/current-system/sw/bin/lsblk").is_file() {
            "/run/current-system/sw/bin/lsblk".into()
        } else {
            "lsblk".into()
        }
    });

    let salida = Command::new(programa)
        .args([
            "--json",
            "--bytes",
            "--output",
            "NAME,SIZE,TYPE,FSTYPE,LABEL,MODEL,TRAN,MOUNTPOINTS",
        ])
        .output()
        .map_err(|error| format!("No pude leer los discos locales.\nDetalle: {error}"))?;

    if !salida.status.success() {
        let detalle = String::from_utf8_lossy(&salida.stderr).trim().to_string();

        return Err(if detalle.is_empty() {
            "No pude leer los discos locales.".to_string()
        } else {
            format!("No pude leer los discos locales.\nDetalle: {detalle}")
        });
    }

    leer_json(&salida.stdout)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn omite_el_disco_del_sistema_y_muestra_el_secundario() {
        let datos = br#"{
          "blockdevices": [
            {
              "size": "1000000000000",
              "type": "disk",
              "model": "NVME-SISTEMA",
              "tran": "nvme",
              "mountpoints": [null],
              "children": [
                {
                  "size": "900000000000",
                  "type": "part",
                  "fstype": "btrfs",
                  "label": null,
                  "mountpoints": ["/"]
                }
              ]
            },
            {
              "size": 500107862016,
              "type": "disk",
              "model": "ST3500413AS ",
              "tran": "sata",
              "mountpoints": [null],
              "children": [
                {
                  "size": 500107859968,
                  "type": "part",
                  "fstype": "ntfs",
                  "label": null,
                  "mountpoints": ["/mnt/datos"]
                }
              ]
            }
          ]
        }"#;

        let unidades = leer_json(datos).expect("debería entender lsblk");

        assert_eq!(unidades.len(), 1);
        assert_eq!(unidades[0].nombre, "ST3500413AS · 500 GB");
        assert_eq!(unidades[0].detalle, "SATA · NTFS");
        assert!(!unidades[0].detalle.contains("/mnt/datos"));
    }
}
