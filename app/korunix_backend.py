"""Adaptador de solo lectura entre la interfaz y el motor actual de Korunix.

La interfaz no evalúa Nix ni inspecciona el sistema por su cuenta. Este módulo
consume los contratos JSON que ya publican los scripts del proyecto y transforma
los identificadores internos en valores comprensibles para una persona.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

try:
    from babel import Locale
    from babel.dates import get_timezone_name
except ImportError:  # Las validaciones estructurales no necesitan Babel.
    Locale = None
    get_timezone_name = None


AREAS = ("localization", "hardware", "users", "channel")


class BackendError(RuntimeError):
    """Error comprensible producido al consultar una parte del motor."""


@dataclass(frozen=True)
class Snapshot:
    """Resultado parcial: un área puede fallar sin inutilizar las demás."""

    root: Path
    data: dict[str, dict[str, Any]] = field(default_factory=dict)
    errors: dict[str, str] = field(default_factory=dict)

    @property
    def available(self) -> tuple[str, ...]:
        return tuple(area for area in AREAS if area in self.data)

    @property
    def complete(self) -> bool:
        return not self.errors and all(area in self.data for area in AREAS)


def _is_project_root(path: Path) -> bool:
    return (
        (path / "flake.nix").is_file()
        and (path / "Cargo.toml").is_file()
        and (path / "sistema" / "programa" / "principal.rs").is_file()
        and (path / "spec.md").is_file()
    )


def find_project_root() -> Path:
    """Encuentra el checkout sin fijar una ruta personal dentro de la app."""

    candidates: list[Path] = []
    configured = os.environ.get("KORUNIX_ROOT")

    if configured:
        candidates.append(Path(configured).expanduser())

    candidates.extend([Path.cwd(), *Path.cwd().parents])
    candidates.append(Path.home() / ".korunix")
    candidates.extend(Path(__file__).resolve().parents)

    seen: set[Path] = set()
    for candidate in candidates:
        resolved = candidate.resolve()
        if resolved in seen:
            continue
        seen.add(resolved)

        if _is_project_root(resolved):
            return resolved

    raise BackendError(
        "No encontramos la configuración de Korunix en este equipo."
    )


def _engine_command(root: Path, *arguments: str) -> list[str]:
    configured = os.environ.get("KORUNIX_MOTOR_BIN")
    if configured:
        return [configured, *arguments]

    built = root / "target" / "debug" / "korunix"
    if built.is_file() and os.access(built, os.X_OK):
        return [str(built), *arguments]

    cargo = shutil.which("cargo")
    if cargo:
        return [
            cargo,
            "run",
            "--quiet",
            "--locked",
            "--bin",
            "korunix",
            "--",
            *arguments,
        ]

    raise BackendError("No encontramos el motor Rust de Korunix.")


def _run_json(root: Path, area: str, timeout: int = 180) -> dict[str, Any]:
    if area not in AREAS:
        raise ValueError(f"Área desconocida: {area}")

    command = _engine_command(root, area, "--json")
    environment = os.environ.copy()
    environment.setdefault("KORUNIX_ROOT", str(root))

    try:
        result = subprocess.run(
            command,
            cwd=root,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        raise BackendError(
            f"La consulta de {area} tardó demasiado y fue detenida."
        ) from error
    except OSError as error:
        raise BackendError(
            f"No pudimos iniciar la consulta de {area}."
        ) from error

    if result.returncode != 0:
        detail = result.stderr.strip().splitlines()
        last_line = detail[-1] if detail else "La consulta no devolvió detalles."
        raise BackendError(f"No pudimos leer {area}: {last_line}")

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise BackendError(
            f"{area} devolvió datos que Korunix no reconoce."
        ) from error

    if not isinstance(payload, dict):
        raise BackendError(
            f"{area} no devolvió un objeto de estado válido."
        )

    return payload

def prepare_channel(
    root: Path,
    target: str,
    timeout: int = 600,
) -> dict[str, Any]:
    if target not in {"stable", "unstable"}:
        raise ValueError(f"Canal desconocido: {target}")

    command = _engine_command(root, "channel", target, "--yes")
    environment = os.environ.copy()
    environment.setdefault("KORUNIX_ROOT", str(root))

    try:
        result = subprocess.run(
            command,
            cwd=root,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        raise BackendError(
            "La preparación del canal tardó demasiado y fue detenida."
        ) from error
    except OSError as error:
        raise BackendError(
            "No pudimos iniciar la preparación del canal."
        ) from error

    if result.returncode != 0:
        detail = [
            *result.stderr.strip().splitlines(),
            *result.stdout.strip().splitlines(),
        ]
        last_line = (
            detail[-1]
            if detail
            else "La operación no devolvió detalles."
        )
        raise BackendError(
            f"No pudimos preparar {target}: {last_line}"
        )

    return _run_json(root, "channel")

def load_snapshot(root: Path | None = None) -> Snapshot:
    """Carga las áreas en paralelo para no congelar la ventana."""

    project_root = root or find_project_root()
    data: dict[str, dict[str, Any]] = {}
    errors: dict[str, str] = {}

    with ThreadPoolExecutor(max_workers=len(AREAS)) as executor:
        futures = {
            executor.submit(_run_json, project_root, area): area for area in AREAS
        }

        for future in as_completed(futures):
            area = futures[future]
            try:
                data[area] = future.result()
            except BackendError as error:
                errors[area] = str(error)

    return Snapshot(root=project_root, data=data, errors=errors)


def _babel_locale(language: str) -> Any | None:
    if Locale is None:
        return None

    try:
        return Locale.parse(language, sep="-")
    except (ValueError, TypeError):
        return None


_LANGUAGES = {
    "es": {"es": "Español", "en": "Spanish", "hu": "Spanyol"},
    "en": {"es": "Inglés", "en": "English", "hu": "Angol"},
    "hu": {"es": "Húngaro", "en": "Hungarian", "hu": "Magyar"},
}

_REGIONS = {
    "PE": {"es": "Perú", "en": "Peru", "hu": "Peru"},
    "ES": {"es": "España", "en": "Spain", "hu": "Spanyolország"},
    "US": {"es": "Estados Unidos", "en": "United States", "hu": "Egyesült Államok"},
    "HU": {"es": "Hungría", "en": "Hungary", "hu": "Magyarország"},
}

_TIME_ZONES = {
    "America/Lima": {"es": "Hora de Lima", "en": "Lima time", "hu": "Limai idő"},
    "Europe/Madrid": {"es": "Hora de Madrid", "en": "Madrid time", "hu": "Madridi idő"},
    "Europe/Budapest": {"es": "Hora de Budapest", "en": "Budapest time", "hu": "Budapesti idő"},
}

_KEYBOARDS = {
    "es": {
        "es": "Español — España",
        "en": "Spanish — Spain",
        "hu": "Spanyol — Spanyolország",
    },
    "latam": {
        "es": "Español — Latinoamérica",
        "en": "Spanish — Latin America",
        "hu": "Spanyol — Latin-Amerika",
    },
    "us": {
        "es": "Inglés — Estados Unidos",
        "en": "English — United States",
        "hu": "Angol — Egyesült Államok",
    },
    "gb": {
        "es": "Inglés — Reino Unido",
        "en": "English — United Kingdom",
        "hu": "Angol — Egyesült Királyság",
    },
    "hu": {
        "es": "Húngaro — Hungría",
        "en": "Hungarian — Hungary",
        "hu": "Magyar — Magyarország",
    },
}


def _initial_upper(value: str) -> str:
    return value[:1].upper() + value[1:] if value else value


def normalize_language(value: str | None) -> str:
    if not value:
        return "es"

    normalized = value.split(":", 1)[0].split(".", 1)[0].split("@", 1)[0]
    return normalized.replace("_", "-").split("-", 1)[0].lower() or "es"


def human_language(code: str | None, display_language: str) -> str:
    normalized = normalize_language(code)
    locale = _babel_locale(display_language)

    if locale is not None:
        name = locale.languages.get(normalized)
        if name:
            return _initial_upper(str(name))

    return _LANGUAGES.get(normalized, {}).get(display_language, normalized.upper())


def human_region(code: str | None, display_language: str) -> str:
    normalized = (code or "").upper()
    locale = _babel_locale(display_language)

    if locale is not None:
        name = locale.territories.get(normalized)
        if name:
            return str(name)

    fallback = _REGIONS.get(normalized, {}).get(display_language)
    return fallback or (normalized if normalized else "—")


def human_locale(
    language: str | None,
    region: str | None,
    display_language: str,
) -> str:
    language_code = normalize_language(language)
    region_code = (region or "").upper()

    if Locale is not None and region_code:
        try:
            locale = Locale(language_code, region_code)
            return _initial_upper(str(locale.get_display_name(display_language)))
        except (ValueError, TypeError):
            pass

    language_name = human_language(language_code, display_language)
    region_name = human_region(region_code, display_language)
    return f"{language_name} ({region_name})"


def human_time_zone(zone: str | None, display_language: str) -> str:
    if not zone:
        return "—"

    fallback = _TIME_ZONES.get(zone, {}).get(display_language)
    if fallback:
        return fallback

    if get_timezone_name is not None:
        try:
            name = get_timezone_name(ZoneInfo(zone), locale=display_language)
            if name and "/" not in name:
                return _initial_upper(str(name))
        except (LookupError, ValueError, ZoneInfoNotFoundError):
            pass

    city = zone.rsplit("/", 1)[-1].replace("_", " ")
    return city if city else "—"


def human_machine_type(machine_type: str | None, language: str) -> str:
    values = {
        "desktop": {
            "es": "Equipo de escritorio",
            "en": "Desktop computer",
            "hu": "Asztali számítógép",
        },
        "laptop": {"es": "Portátil", "en": "Laptop", "hu": "Laptop"},
        "all-in-one": {"es": "Todo en uno", "en": "All-in-one", "hu": "All-in-one számítógép"},
        "server": {"es": "Servidor", "en": "Server", "hu": "Kiszolgáló"},
        "unknown": {
            "es": "Tipo no determinado",
            "en": "Type not detected",
            "hu": "Ismeretlen típus",
        },
    }
    return values.get(machine_type or "unknown", values["unknown"]).get(
        language, values["unknown"]["es"]
    )


def human_architecture(value: str | None) -> str:
    architectures = {
        "x86_64-linux": "64 bits · Intel/AMD",
        "aarch64-linux": "64 bits · ARM",
        "i686-linux": "32 bits · Intel/AMD",
    }
    if value in architectures:
        return architectures[value]

    compact = (value or "").removesuffix("-linux").replace("_", " ")
    return compact or "—"


_PCI_IDENTIFIER = re.compile(r"\[[0-9a-f]{4}:[0-9a-f]{4}\]", re.IGNORECASE)
_PCI_REVISION = re.compile(r"\s*\(rev\s+[^)]+\)\s*$", re.IGNORECASE)


def _pci_description(value: str) -> str:
    """Retira metadatos de inventario sin alterar el dato fuente."""

    _device_class, separator, description = value.partition(": ")
    visible = description if separator else value
    visible = _PCI_IDENTIFIER.sub("", visible)
    visible = _PCI_REVISION.sub("", visible)
    visible = visible.replace("Advanced Micro Devices, Inc. [AMD/ATI]", "AMD")
    visible = visible.replace("NVIDIA Corporation", "NVIDIA")
    visible = visible.replace("Intel Corporation", "Intel")
    visible = visible.replace("Realtek Semiconductor Co., Ltd.", "Realtek")
    visible = visible.replace("Qualcomm Atheros", "Qualcomm")
    visible = re.sub(r"\[([^]]+)]", r"\1", visible)
    visible = re.sub(
        r"\b(?:PCI Express|compatible controller|controller|adapter)\b",
        "",
        visible,
        flags=re.IGNORECASE,
    )
    visible = re.sub(r"\s+", " ", visible).strip(" ,-_")
    return visible or "—"


def human_graphics_device(value: str) -> str:
    """Prefiere la familia comercial sobre clases, IDs y nombres de silicio."""

    radeon = re.search(r"Radeon\s+([^/\]]+)", value, re.IGNORECASE)
    if radeon:
        family = re.sub(
            r"\b(?:Series|Mobile)\b",
            "",
            radeon.group(1),
            flags=re.IGNORECASE,
        )
        family = re.sub(r"\s+", " ", family).strip()
        return f"AMD Radeon {family}" if family else "AMD Radeon"

    geforce = re.search(r"GeForce\s+([^/\]]+)", value, re.IGNORECASE)
    if geforce:
        return f"NVIDIA GeForce {geforce.group(1).strip()}"

    intel = re.search(
        r"(?:UHD|Iris|Arc)\s+(?:Graphics\s+)?([^/\]]+)",
        value,
        re.IGNORECASE,
    )
    if intel:
        prefix = re.search(r"(?:UHD|Iris|Arc)", value, re.IGNORECASE)
        family = prefix.group(0) if prefix else ""
        family = family.upper() if family.lower() == "uhd" else family.title()
        model = intel.group(1).strip()
        return f"Intel {family} Graphics {model}".strip()

    return _pci_description(value)


def human_network_device(value: str) -> str:
    """Presenta la capacidad y el fabricante; el modelo PCI queda avanzado."""

    device_class = value.partition(": ")[0].lower()
    lowered = value.lower()
    connection = (
        "Ethernet"
        if "ethernet" in device_class
        else "Wi-Fi"
        if "network" in device_class or "wireless" in device_class
        else "Red"
    )

    vendors = (
        ("realtek", "Realtek"),
        ("intel", "Intel"),
        ("qualcomm", "Qualcomm"),
        ("broadcom", "Broadcom"),
        ("mediatek", "MediaTek"),
    )
    for marker, vendor in vendors:
        if marker in lowered:
            return f"{connection} {vendor}"

    return _pci_description(value)


def human_graphics_driver(value: str, language: str) -> str:
    names = {
        "amdgpu": {
            "es": "Controlador gráfico AMD",
            "en": "AMD graphics driver",
            "hu": "AMD grafikus illesztőprogram",
        },
        "i915": {
            "es": "Controlador gráfico Intel",
            "en": "Intel graphics driver",
            "hu": "Intel grafikus illesztőprogram",
        },
        "xe": {
            "es": "Controlador gráfico Intel",
            "en": "Intel graphics driver",
            "hu": "Intel grafikus illesztőprogram",
        },
        "nvidia": {
            "es": "Controlador gráfico NVIDIA",
            "en": "NVIDIA graphics driver",
            "hu": "NVIDIA grafikus illesztőprogram",
        },
        "nouveau": {
            "es": "Controlador gráfico abierto para NVIDIA",
            "en": "Open NVIDIA graphics driver",
            "hu": "Nyílt NVIDIA grafikus illesztőprogram",
        },
    }
    translated = names.get(value.lower())
    return translated.get(language, translated["es"]) if translated else value


def human_desktop(value: str | None) -> str:
    if not value:
        return "—"

    names = {
        "niri": "Niri",
        "hyprland": "Hyprland",
        "hyprland-uwsm": "Hyprland",
        "kde": "KDE Plasma",
        "plasma": "KDE Plasma",
        "cinnamon": "Cinnamon",
        "gnome": "GNOME",
    }
    parts = [part for part in value.replace(";", ":").split(":") if part]
    resolved = [names.get(part.lower(), part) for part in parts]
    return " · ".join(dict.fromkeys(resolved)) or "—"


def format_memory(value: Any) -> str:
    try:
        bytes_total = int(value)
    except (TypeError, ValueError):
        return "—"

    return f"{bytes_total / 1_073_741_824:.1f} GiB"


def _clean_strings(values: Iterable[Any]) -> list[str]:
    return [str(value) for value in values if value not in (None, "")]


def human_keyboards(
    layout_value: str | None,
    declared_names: Iterable[Any],
    display_language: str,
) -> list[str]:
    layouts = [item.strip() for item in (layout_value or "").split(",") if item.strip()]
    declared = _clean_strings(declared_names)
    names: list[str] = []

    for index, layout in enumerate(layouts):
        translated = _KEYBOARDS.get(layout, {}).get(display_language)
        if translated:
            names.append(translated)
        elif index < len(declared):
            names.append(declared[index])
        else:
            names.append(layout.upper())

    return names or declared or ["—"]


def present_localization(data: dict[str, Any], language: str) -> dict[str, Any]:
    declared = data.get("declared", {})
    formats = declared.get("formats", {})
    keyboard = declared.get("keyboard", {})
    derived_keyboard = data.get("derived", {}).get("keyboard", {})
    runtime = data.get("runtime", {})
    input_method = data.get("inputMethod", {}).get("nixos", {})

    display_names = human_keyboards(
        derived_keyboard.get("layout"),
        keyboard.get("displayNames", []),
        language,
    )

    return {
        "systemLanguage": human_language(declared.get("systemLanguage"), language),
        "region": human_region(declared.get("region"), language),
        "formats": human_locale(
            formats.get("language"), formats.get("region"), language
        ),
        "timeZone": human_time_zone(declared.get("timeZone"), language),
        "keyboards": display_names,
        "inputMethod": human_input_method(input_method.get("type"), language),
        "desktop": human_desktop(runtime.get("desktop")),
        "contradictions": list(data.get("contradictions", [])),
        "supportedInterfaceLanguages": _clean_strings(
            data.get("noctalia", {}).get("supportedLanguages", [])
        ),
    }


def human_input_method(value: str | None, language: str) -> str:
    if value == "fcitx5":
        return "Fcitx 5"
    if value == "ibus":
        return "IBus"

    messages = {
        "es": "No se necesita uno adicional",
        "en": "No additional method is needed",
        "hu": "Nincs szükség további beviteli módra",
    }
    return messages.get(language, messages["es"])


def present_hardware(data: dict[str, Any], language: str) -> dict[str, Any]:
    machine = data.get("machine", {})
    platform = data.get("platform", {})
    firmware = data.get("firmware", {})
    cpu = data.get("cpu", {})

    firmware_name = str(firmware.get("detected") or "").upper() or "—"
    return {
        "type": human_machine_type(machine.get("type"), language),
        "vendor": machine.get("vendor") or "—",
        "model": machine.get("model") or "—",
        "architecture": human_architecture(platform.get("detected")),
        "architectureMatches": bool(platform.get("matches")),
        "firmware": firmware_name,
        "firmwareMatches": bool(firmware.get("matches")),
        "processor": cpu.get("model") or "—",
        "logicalProcessors": cpu.get("logicalProcessors") or 0,
        "memory": format_memory(data.get("memory", {}).get("bytes")),
        "graphics": [
            human_graphics_device(device)
            for device in _clean_strings(data.get("graphics", []))
        ]
        or ["—"],
        "graphicsDrivers": [
            human_graphics_driver(driver, language)
            for driver in _clean_strings(data.get("graphicsDrivers", []))
        ],
        "network": [
            human_network_device(device)
            for device in _clean_strings(data.get("network", []))
        ],
    }


def present_channel(
    data: dict[str, Any],
    language: str,
) -> dict[str, Any]:
    """Presenta el canal en lenguaje humano y conserva los detalles técnicos."""

    channel_id = str(data.get("declared") or "")
    effective = str(data.get("effective") or "")
    sources = data.get("sources", {})

    options: list[dict[str, Any]] = []

    for option in data.get("options", []):
        if not isinstance(option, dict):
            continue

        option_id = str(option.get("id") or "")
        labels = option.get("labels", {})
        descriptions = option.get("descriptions", {})

        if not isinstance(labels, dict) or not isinstance(descriptions, dict):
            continue

        label = (
            labels.get(language)
            or labels.get("es")
            or option_id
            or "—"
        )
        description = (
            descriptions.get(language)
            or descriptions.get("es")
            or ""
        )

        options.append(
            {
                "id": option_id,
                "label": label,
                "description": description,
            }
        )

    current_option = next(
        (
            option
            for option in options
            if option.get("id") == channel_id
        ),
        None,
    )

    raw_version = str(data.get("nixosVersion") or "—")
    version_parts = raw_version.split(".")

    if (
        len(version_parts) >= 2
        and version_parts[0].isdigit()
        and version_parts[1].isdigit()
    ):
        short_version = ".".join(version_parts[:2])
    else:
        short_version = raw_version

    return {
        "id": channel_id,
        "label": (
            current_option.get("label")
            if current_option
            else channel_id or "—"
        ),
        "description": (
            current_option.get("description")
            if current_option
            else ""
        ),
        "effectiveMatches": effective == channel_id,
        "nixosVersion": raw_version,
        "nixosVersionShort": short_version,
        "stateVersion": data.get("stateVersion") or "—",
        "nixpkgs": sources.get("nixpkgs") or "—",
        "aagl": sources.get("aagl") or "—",
        "options": options,
    }

def present_users(data: dict[str, Any]) -> dict[str, Any]:
    accounts = []
    for account in data.get("accounts", []):
        accounts.append(
            {
                "name": account.get("displayName") or account.get("accountName") or "—",
                "account": account.get("accountName") or "—",
                "administrator": bool(account.get("administrator")),
                "status": account.get("status") or "adoptable",
            }
        )

    summary = data.get("summary", {})
    return {
        "accounts": accounts,
        "humanAccounts": int(summary.get("humanAccounts", len(accounts))),
        "adoptedAccounts": int(summary.get("adoptedAccounts", 0)),
        "detectedAdministrators": int(summary.get("detectedAdministrators", 0)),
    }
