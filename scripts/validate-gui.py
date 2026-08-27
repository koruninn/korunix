#!/usr/bin/env python3
"""Validación rápida y sin interfaz gráfica del primer centro de control."""

from __future__ import annotations

import ast
import json
import os
import sys
from pathlib import Path
from typing import Any
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "app"
sys.path.insert(0, str(APP))

from korunix_backend import (  # noqa: E402
    BackendError,
    find_project_root,
    human_architecture,
    normalize_language,
    present_channel,
    present_hardware,
    present_localization,
    present_users,
    load_snapshot,
)
from korunix_i18n import CATALOGS, Translator  # noqa: E402


def flatten_strings(value: Any) -> list[str]:
    if isinstance(value, dict):
        strings: list[str] = []
        for child in value.values():
            strings.extend(flatten_strings(child))
        return strings
    if isinstance(value, list):
        strings = []
        for child in value:
            strings.extend(flatten_strings(child))
        return strings
    return [value] if isinstance(value, str) else []


def validate_python() -> None:
    for path in sorted(APP.glob("*.py")):
        source = path.read_text(encoding="utf-8")
        ast.parse(source, filename=str(path))


def validate_catalogs() -> None:
    canonical = set(CATALOGS["es"])
    for language, catalog in CATALOGS.items():
        missing = canonical - set(catalog)
        extra = set(catalog) - canonical
        if missing or extra:
            raise SystemExit(
                f"El catálogo {language} no coincide: faltan {sorted(missing)}, "
                f"sobran {sorted(extra)}."
            )

    for language in CATALOGS:
        translator = Translator(language)
        if translator.text("summary.title") == "summary.title":
            raise SystemExit(f"El catálogo {language} no traduce el resumen.")

    tree = ast.parse((APP / "korunix.py").read_text(encoding="utf-8"))
    used: set[str] = set()
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call) or not node.args:
            continue
        if not isinstance(node.func, ast.Attribute) or node.func.attr != "text":
            continue
        for child in ast.walk(node.args[0]):
            if isinstance(child, ast.Constant) and isinstance(child.value, str):
                if "." in child.value:
                    used.add(child.value)

    missing_from_source = used - canonical
    if missing_from_source:
        raise SystemExit(
            "La interfaz usa textos que no existen en español: "
            f"{sorted(missing_from_source)}."
        )


def validate_partial_degradation() -> None:
    def fake_query(_root: Path, area: str, timeout: int = 180) -> dict[str, Any]:
        del timeout
        if area == "hardware":
            raise BackendError("Hardware temporalmente no disponible.")
        return {"schemaVersion": 1, "area": area}

    with patch("korunix_backend._run_json", side_effect=fake_query):
        snapshot = load_snapshot(ROOT)

    if set(snapshot.available) != {"localization", "users", "channel"}:
        raise SystemExit("Un fallo parcial ocultó áreas que seguían disponibles.")
    if "hardware" not in snapshot.errors:
        raise SystemExit("El fallo parcial de hardware no quedó registrado.")


def validate_human_presentation() -> None:
    localization = {
        "declared": {
            "systemLanguage": "es",
            "region": "PE",
            "formats": {"language": "es", "region": "PE"},
            "timeZone": "America/Lima",
            "keyboard": {
                "displayNames": [
                    "Español — España",
                    "Español — Latinoamérica",
                ]
            },
        },
        "runtime": {"desktop": "niri"},
        "inputMethod": {"nixos": {"type": "none"}},
        "noctalia": {"supportedLanguages": ["es", "en", "hu"]},
        "contradictions": [],
        "derived": {
            "systemLocale": "es_PE.UTF-8",
            "formatLocale": "es_PE.UTF-8",
            "keyboard": {
                "layout": "es,latam",
                "options": "grp:alt_shift_toggle",
            },
        },
    }

    hardware = {
        "machine": {"type": "desktop", "vendor": "Korunix", "model": "Prueba"},
        "platform": {"detected": "x86_64-linux", "matches": True},
        "firmware": {"detected": "uefi", "matches": True},
        "cpu": {"model": "CPU de prueba", "logicalProcessors": 8},
        "memory": {"bytes": 17_179_869_184},
        "graphics": [
            "VGA compatible controller [0300]: Advanced Micro Devices, Inc. "
            "[AMD/ATI] Cezanne [Radeon Vega Series / Radeon Vega Mobile Series] "
            "[1002:1638] (rev c9)"
        ],
        "graphicsDrivers": ["amdgpu"],
        "network": [
            "Ethernet controller [0200]: Realtek Semiconductor Co., Ltd. "
            "RTL8111/8168/8211/8411 PCI Express Gigabit Ethernet Controller "
            "[10ec:8168] (rev 15)"
        ],
    }

    channel = {
        "schemaVersion": 1,
        "hostId": "prueba",
        "declared": "stable",
        "effective": "stable",
        "nixosVersion": "26.05",
        "stateVersion": "26.05",
        "sources": {
            "nixpkgs": "nixos-26.05",
            "aagl": "release-26.05",
        },
        "options": [
            {
                "id": "stable",
                "labels": {
                    "es": "Estable",
                    "en": "Stable",
                    "hu": "Stabil",
                },
                "descriptions": {
                    "es": "Prioriza estabilidad.",
                    "en": "Prioritizes stability.",
                    "hu": "A stabilitást részesíti előnyben.",
                },
            },
            {
                "id": "unstable",
                "labels": {
                    "es": "Inestable",
                    "en": "Unstable",
                    "hu": "Instabil",
                },
                "descriptions": {
                    "es": "Prioriza software reciente.",
                    "en": "Prioritizes newer software.",
                    "hu": "Az újabb szoftvereket részesíti előnyben.",
                },
            },
        ],
    }

    users = {
        "accounts": [
            {
                "displayName": "Persona de prueba",
                "accountName": "persona",
                "administrator": True,
                "status": "adopted",
            }
        ],
        "summary": {
            "humanAccounts": 1,
            "adoptedAccounts": 1,
            "detectedAdministrators": 1,
        },
    }

    for language in CATALOGS:
        presented = present_localization(localization, language)
        visible = "\n".join(flatten_strings(presented))
        forbidden = ("es_PE.UTF-8", "America/Lima", "grp:alt_shift_toggle")
        for technical in forbidden:
            if technical in visible:
                raise SystemExit(
                    f"El valor técnico {technical} se filtró en la vista {language}."
                )

        if "/" in presented["timeZone"]:
            raise SystemExit("La zona horaria visible conserva un identificador interno.")

        if language != "es" and any("Español" in name for name in presented["keyboards"]):
            raise SystemExit(f"Los teclados no reaccionan al idioma {language}.")

        hardware_view = present_hardware(hardware, language)
        if hardware_view["architecture"] == "x86_64-linux":
            raise SystemExit("La arquitectura visible conserva el triplete de Nix.")

        hardware_visible = "\n".join(flatten_strings(hardware_view))
        forbidden_hardware = (
            "[0300]",
            "[0200]",
            "[1002:1638]",
            "[10ec:8168]",
            "(rev c9)",
            "(rev 15)",
            "amdgpu",
            "compatible controller",
        )
        for technical in forbidden_hardware:
            if technical.lower() in hardware_visible.lower():
                raise SystemExit(
                    f"El valor técnico {technical} se filtró en hardware {language}."
                )

        if hardware_view["graphics"] != ["AMD Radeon Vega"]:
            raise SystemExit("La GPU no se presenta mediante su familia comercial.")
        if hardware_view["network"] != ["Ethernet Realtek"]:
            raise SystemExit("La red no se presenta mediante su capacidad y fabricante.")

        channel_view = present_channel(channel, language)
        if channel_view["id"] != "stable":
            raise SystemExit("La presentación del canal perdió su identificador.")

        if channel_view["label"] == "stable":
            raise SystemExit(
                f"El canal no se presenta en lenguaje humano para {language}."
            )

        if not channel_view["effectiveMatches"]:
            raise SystemExit("El canal declarado y el efectivo deberían coincidir.")

        if channel_view["nixosVersionShort"] != "26.05":
            raise SystemExit(
                "La versión humana de NixOS conserva la revisión técnica."
            )

        if len(channel_view["options"]) != 2:
            raise SystemExit(
                "La GUI no recibió las dos opciones de canal."
            )

        if not all(
            option["description"]
            for option in channel_view["options"]
        ):
            raise SystemExit(
                f"Falta la descripción humana de un canal para {language}."
            )

    people = present_users(users)
    if people["accounts"][0]["name"] != "Persona de prueba":
        raise SystemExit("La presentación de personas perdió el nombre humano.")


def validate_structure() -> None:
    expected = (
        APP / "korunix.py",
        APP / "korunix_backend.py",
        APP / "korunix_i18n.py",
        APP / "style.css",
        APP / "io.github.koruninn.Korunix.desktop",
    )
    for path in expected:
        if not path.is_file():
            raise SystemExit(f"Falta {path.relative_to(ROOT)}.")

    desktop = (APP / "io.github.koruninn.Korunix.desktop").read_text(
        encoding="utf-8"
    )
    if "Exec=korunix" not in desktop or "Terminal=false" not in desktop:
        raise SystemExit("El lanzador gráfico no apunta al ejecutable de Korunix.")

    flake = (ROOT / "flake.nix").read_text(encoding="utf-8")
    for required in ("korunixGuiFor", "pythonPackages.pygobject3", "pkgs.libadwaita"):
        if required not in flake:
            raise SystemExit(f"flake.nix no contiene {required}.")

    interface = (APP / "korunix.py").read_text(encoding="utf-8")
    for required in (
        "Adw.BreakpointCondition.parse",
        'breakpoint.add_setter(self.split_view, "collapsed", True)',
        "self.add_breakpoint(breakpoint)",
        "Gtk.SearchEntry",
        "Adw.ComboRow",
        "def _on_search_changed",
        "def _build_updates_page",
        "prepare_channel(root, target)",
        '"updates"',
        "korunix-status-row",
    ):
        if required not in interface:
            raise SystemExit(f"La ventana adaptable no contiene {required}.")

    if 'self.text("channels.nixpkgs")' in interface:
        raise SystemExit(
            "La vista normal vuelve a mostrar la referencia de Nixpkgs."
        )

    if 'self.text("channels.aagl")' in interface:
        raise SystemExit(
            "La vista normal vuelve a mostrar la referencia de AAGL."
        )

    style = (APP / "style.css").read_text(encoding="utf-8")
    for required in (".korunix-search", ".korunix-status-row"):
        if required not in style:
            raise SystemExit(f"El sistema visual no contiene {required}.")

    backend = (APP / "korunix_backend.py").read_text(
        encoding="utf-8"
    )
    if "def _engine_command" not in backend:
        raise SystemExit("La GUI no conoce el motor Rust.")
    if 'root / "scripts" / "korunix"' in backend:
        raise SystemExit("La GUI volvió a invocar Bash.")
    if "KORUNIX_MOTOR_BIN" not in backend:
        raise SystemExit(
            "La GUI no acepta el ejecutable Rust empaquetado."
        )


def main() -> None:
    os.chdir(ROOT)
    validate_python()
    validate_catalogs()
    validate_partial_degradation()
    validate_human_presentation()
    validate_structure()

    if find_project_root() != ROOT:
        raise SystemExit("La interfaz no reconoce la raíz del checkout actual.")

    if normalize_language("es_PE.UTF-8") != "es":
        raise SystemExit("La detección de idioma no normaliza locales de NixOS.")

    if human_architecture("x86_64-linux") == "x86_64-linux":
        raise SystemExit("La arquitectura no se presenta en lenguaje humano.")

    print(
        json.dumps(
            {
                "python": "correcto",
                "catalogs": sorted(CATALOGS),
                "humanPresentation": "correcta",
                "projectRoot": str(ROOT),
            },
            ensure_ascii=False,
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
