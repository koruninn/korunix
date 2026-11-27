# Cambiar las rutas sin reemplazar las configuraciones completas.
import json
import os
import re
import shutil
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1])
mode = sys.argv[2]

def warn(msg):
    print("NixDeck: " + msg, file=sys.stderr)

def write(path, data):
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        if path.read_text() == data:
            return
        backup = path.with_name(path.name + ".nixdeck-respaldo")
        if not backup.exists():
            shutil.copy2(path, backup)
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as f:
        f.write(data)
        tmp = f.name
    os.replace(tmp, path)

def settings(path, changes, toml=False):
    old = path.read_text() if path.exists() else ""
    lines = old.splitlines()
    for section, values in changes.items():
        header = "[" + section + "]"
        indexes = [i for i, line in enumerate(lines) if line.strip() == header]
        if len(indexes) > 1:
            raise ValueError("Sección duplicada en " + str(path))
        if not indexes:
            if lines and lines[-1]:
                lines.append("")
            lines.append(header)
            indexes = [len(lines)-1]
        i = indexes[0] + 1
        end = next((n for n in range(i, len(lines)) if lines[n].lstrip().startswith("[")), len(lines))
        for key, value in values.items():
            text = json.dumps(str(value), ensure_ascii=False) if toml and isinstance(value, (str, Path)) else ("true" if value is True else "false" if value is False else str(value))
            matches = [n for n in range(i, end) if re.match(r"^\s*" + re.escape(key) + r"\s*=", lines[n])]
            if len(matches) > 1:
                raise ValueError("Clave duplicada: " + key)
            if matches:
                lines[matches[0]] = key + " = " + text
            else:
                lines.insert(end, key + " = " + text)
                end += 1
    new = "\n".join(lines) + "\n"
    if toml:
        import tomllib
        tomllib.loads(new)
    write(path, new)

def dirs(*names):
    for name in names:
        (root / name).mkdir(parents=True, exist_ok=True)

def first_file(folder, names):
    for name in names:
        p = root / folder / name
        if p.is_file():
            return p
    return None

def mgba():
    dirs("saves/mgba", "states/mgba", "screenshots/mgba", "mods/mgba/cheats", "mods/mgba/patches")
    fields = {
        "savegamePath": root/"saves/mgba",
        "savestatePath": root/"states/mgba",
        "screenshotPath": root/"screenshots/mgba",
        "cheatsPath": root/"mods/mgba/cheats",
        "patchPath": root/"mods/mgba/patches",
    }
    for key, folder, names in [
        ("gba.bios", "bios/gba", ["gba_bios.bin", "gba.bin", "bios.bin"]),
    ]:
        p = first_file(folder, names)
        if p:
            fields[key] = p
            fields["useBios"] = 1
    settings(root/"emulators/mgba/config/mgba/config.ini", {"ports.qt": fields})

def melonds():
    dirs("saves/nds", "states/nds", "mods/nds/cheats")
    cfg = root/"emulators/melonds/config/melonDS/melonDS.toml"
    sections = {"Instance0": {"SaveFilePath": root/"saves/nds", "SavestatePath": root/"states/nds", "CheatFilePath": root/"mods/nds/cheats"}}
    for section, folder, files in [
        ("DS", "bios/nds", {"BIOS7Path": ["bios7.bin"], "BIOS9Path": ["bios9.bin"], "FirmwarePath": ["firmware.bin"]}),
        ("DSi", "bios/dsi", {"BIOS7Path": ["dsi_bios7.bin", "bios7.bin"], "BIOS9Path": ["dsi_bios9.bin", "bios9.bin"], "FirmwarePath": ["dsi_firmware.bin", "firmware.bin"], "NANDPath": ["dsi_nand.bin", "nand.bin"]}),
    ]:
        found = {k: first_file(folder, n) for k,n in files.items()}
        if all(found.values()):
            sections[section] = found
            if section == "DS":
                sections["Emu"] = {"ExternalBIOSEnable": True}
    settings(cfg, sections, toml=True)

def combined_manifest(console):
    # Regenerar la lista de juegos de PS3 y Wii U al abrir o sincronizar Steam ROM Manager.
    import struct
    import xml.etree.ElementTree as ET

    def sfo_title(path, fallback):
        try:
            data = path.read_bytes()
            magic, _, keys, values, count = struct.unpack_from("<4s4I", data)
            if magic != b"\x00PSF":
                return fallback
            for i in range(count):
                offset, fmt, length, _, value = struct.unpack_from("<HHIII", data, 20 + 16*i)
                key = data[keys+offset:].split(b"\0",1)[0]
                if key == b"TITLE" and fmt == 0x0204:
                    title = data[values+value:values+value+length].split(b"\0",1)[0].decode("utf-8", "replace").strip()
                    return title or fallback
        except (OSError, ValueError, struct.error):
            pass
        return fallback

    def wiiu_title(game):
        try:
            meta = ET.parse(game/"meta/meta.xml").getroot()
            for tag in ("longname_es", "longname_en", "shortname_es", "shortname_en"):
                title = meta.findtext(tag)
                if title and title.strip():
                    return title.strip()
        except (OSError, ET.ParseError):
            pass
        return game.name

    def files(folder, suffixes):
        if folder.is_dir():
            return sorted(p for p in folder.rglob("*") if p.is_file() and p.suffix.lower() in suffixes)
        return []

    entries = []
    seen = set()
    emu = "rpcs3" if console == "ps3" else "cemu"
    executable = "/run/current-system/sw/bin/korunix-" + emu
    flags = "--no-gui" if console == "ps3" else "-f -g"

    def add(title, target):
        if not target.is_file():
            return
        key = target.resolve()
        if key in seen:
            return
        seen.add(key)
        quoted = '"' + str(target).replace("\\", "\\\\").replace('"', '\\"') + '"'
        entries.append({"title": title, "target": executable,
                        "startIn": "/run/current-system/sw/bin",
                        "launchOptions": flags + " " + quoted,
                        "appendArgsToExecutable": True})

    folder = root/"roms"/console
    if console == "ps3":
        for game in files(folder, {".iso"}):
            add(game.stem, game)
        for eboot in files(folder, {".bin"}):
            if eboot.name.lower() == "eboot.bin" and eboot.parent.name.upper() == "USRDIR" and eboot.parent.parent.name.upper() == "PS3_GAME":
                game = eboot.parent.parent
                add(sfo_title(game/"PARAM.SFO", game.parent.name), eboot)
        installed = root/"storage/ps3/dev_hdd0/game"
        if installed.is_dir():
            for game in sorted(installed.iterdir()):
                if game.is_dir():
                    for eboot in (game/"USRDIR/EBOOT.BIN", game/"USRDIR/eboot.bin"):
                        add(sfo_title(game/"PARAM.SFO", game.name), eboot)
    else:
        for game in files(folder, {".wua", ".wud", ".wux"}):
            add(game.stem, game)
        # Preferir el ejecutable que está en la carpeta code.
        by_game = {}
        for rpx in files(folder, {".rpx"}):
            game = rpx.parent.parent if rpx.parent.name.lower() == "code" else rpx.parent
            by_game.setdefault(game, []).append(rpx)
        for game, candidates in sorted(by_game.items()):
            add(wiiu_title(game), candidates[0])

    dest = root/"storage/steam-rom-manager/manifests"/console
    dest.mkdir(parents=True, exist_ok=True)
    write(dest/"games.json", json.dumps(entries, ensure_ascii=False, indent=2)+"\n")
    return dest

def srm():
    # Steam conserva su ubicación original.
    home = Path(sys.argv[3])
    dest = root/"storage/steam-rom-manager/userData"
    dest.mkdir(parents=True, exist_ok=True)
    target = dest/"userConfigurations.json"
    # Importar los ajustes anteriores una sola vez, sin sobrescribir.
    if not target.exists():
        conf = Path(os.environ.get("XDG_CONFIG_HOME", str(home/".config")))
        for src in [conf/"Steam ROM Manager/userData", conf/"steam-rom-manager/userData"]:
            if (src/"userConfigurations.json").is_file() and src.resolve() != dest.resolve():
                for item in src.iterdir():
                    out = dest/item.name
                    if not out.exists() and not out.is_symlink():
                        if item.is_dir():
                            shutil.copytree(item, out)
                        elif item.is_file():
                            shutil.copy2(item, out)
                break
    old = json.loads(target.read_text()) if target.exists() else []
    if not isinstance(old, list) or not all(isinstance(x, dict) for x in old):
        raise ValueError("userConfigurations.json no es una lista de parsers")
    steam = home/".steam/steam"
    if not steam.is_dir() and (home/".local/share/Steam").is_dir():
        steam = home/".local/share/Steam"
    # Steam ROM Manager 2.5.44 actualiza el formato v22 por su cuenta.
    # Conservar los identificadores y ajustes existentes.
    specs = [
        (1, "PS1", "psx", "duckstation", "cue|chd|pbp|m3u|iso", '-batch -fullscreen -- "${filePath}"', None),
        (2, "PS2", "ps2", "pcsx2", "iso|chd|cso|zso|gz|bin|mdf|nrg", '-batch -fullscreen -- "${filePath}"', None),
        (3, "PS3", "ps3", "rpcs3", "", '--no-gui "${filePath}"', '**/${title}/PS3_GAME/USRDIR/@(EBOOT.BIN|eboot.bin)'),


        (6, "PSP", "psp", "ppsspp", "iso|cso|chd|pbp", '-f "${filePath}"', None),
        (7, "GameCube", "gc", "dolphin", "iso|gcm|gcz|rvz|wia|ciso|dol", '-b -e "${filePath}"', None),
        (8, "Wii", "wii", "dolphin", "iso|wbfs|rvz|gcz|wia|wad", '-b -e "${filePath}"', None),
        (9, "Wii U", "wiiu", "cemu", "wua|wud|wux", '-f -g "${filePath}"', None),

        (11, "Switch", "switch", "eden", "nsp|xci|nro", '-f -g "${filePath}"', None),
        (12, "3DS", "n3ds", "azahar", "3ds|cci|cxi|app|3dsx", '-f "${filePath}"', None),
        (13, "DS", "nds", "melonds", "nds|dsi", '"${filePath}"', None),


        (16, "GBA", "gba", "mgba", "gba|zip|7z", '-f "${filePath}"', None),
        (17, "Xbox 360", "xbox360", "xenia", "iso|xex", '"${filePath}"', None),
    ]
    image = dict.fromkeys(["long","tall","hero","logo","icon"], "")
    controllers = dict.fromkeys(["ps4","ps5","ps5_edge","xbox360","xboxone","xboxelite","switch_joycon_left","switch_joycon_right","switch_pro","neptune","steamcontroller_gordon"])
    # No reutilizar los identificadores de GB y GBC.
    obsolete = {"176000000000000004", "176000000000000005", "176000000000000010", "176000000000000014", "176000000000000015"}
    old = [p for p in old if str(p.get("parserId")) not in obsolete]
    # Unir los parsers anteriores de PS3 y Wii U sin tocar los demás.
    combined = {"ps3": (3, "PS3", "rpcs3"), "wiiu": (9, "Wii U", "cemu")}
    for console, (number, title, emu) in combined.items():
        pid = str(176000000000000000+number)
        manifest = combined_manifest(console)
        executable = "/run/current-system/sw/bin/korunix-"+emu
        sources = {str(root/"roms"/console)}
        if console == "ps3":
            sources.add(str(root/"storage/ps3/dev_hdd0/game"))
        matches = [item for item in old if str(item.get("parserId")) == pid or
                   (item.get("romDirectory") in sources and (item.get("executable") or {}).get("path") == executable)]
        if matches:
            keep = next((item for item in matches if str(item.get("parserId")) == pid), matches[0])
            # Conservar el identificador y las imágenes.
            old = [item for item in old if item is keep or not any(item is x for x in matches)]
        else:
            keep = {"parserId": pid, "version": 28,
                    "titleModifier": "${fuzzyTitle}", "onlineImageQueries": ["${fuzzyTitle}"],
                    "imagePool": "${fuzzyTitle}", "imageProviders": ["sgdb", "steamCDN"],
                    "userAccounts": {"specifiedAccounts": ["Global"]},
                    "steamInputEnabled": "1", "disabled": False}
            old.append(keep)
        keep.update({"configTitle": title, "steamCategories": [title],
                     "parserType": "Manual", "parserInputs": {"manualManifests": str(manifest)},
                     "steamDirectory": str(steam), "romDirectory": "",
                     "executable": {"path": "", "shortcutPassthrough": False, "appendArgsToExecutable": False},
                     "executableArgs": "", "executableModifier": "", "startInDirectory": ""})
    existing = {str(p.get("parserId")): p for p in old}
    for n,name,console,emu,extensions,args,glob in specs:
        if console in combined:
            continue
        pid = str(176000000000000000+n)
        if pid in existing:
            existing[pid]["configTitle"] = name
            existing[pid]["steamCategories"] = [name]
            continue
        folder = root/"roms"/console if console else root/"storage/ps3/dev_hdd0/game"
        folder.mkdir(parents=True,exist_ok=True)
        executable = "/run/current-system/sw/bin/korunix-"+emu
        if glob is None:
            exts = extensions.split("|")
            glob = "**/${title}@(" + "|".join("."+x for e in exts for x in (e,e.upper())) + ")"
        equivalent = next((p for p in old if p.get("romDirectory") == str(folder) and (p.get("executable") or {}).get("path") == executable and (console not in ("ps3", "wiiu", None) or (p.get("parserInputs") or {}).get("glob") == glob)), None)
        if equivalent is not None:
            equivalent["configTitle"] = name
            equivalent["steamCategories"] = [name]
            continue
        old.append({
            "parserType":"Glob", "configTitle":name,
            "steamDirectory":str(steam), "romDirectory":str(folder),
            "steamCategories":[name], "executableArgs":args,
            "executableModifier":'"${exePath}"', "startInDirectory":"",
            "titleModifier":"${fuzzyTitle}", "steamInputEnabled":"1",
            "imageProviders":["sgdb","steamCDN"], "onlineImageQueries":"${${fuzzyTitle}}",
            "imagePool":"${fuzzyTitle}", "drmProtect":False, "disabled":False,
            "userAccounts":{"specifiedAccounts":["Global"]},
            "executable":{"path":executable,"shortcutPassthrough":False,"appendArgsToExecutable":True},
            "parserInputs":{"glob":glob},
            "titleFromVariable":{"limitToGroups":"","caseInsensitiveVariables":False,"skipFileIfVariableWasNotFound":False},
            "fuzzyMatch":{"replaceDiacritics":True,"removeCharacters":True,"removeBrackets":True},
            "controllers":controllers, "imageProviderAPIs":{"sgdb":{"nsfw":False,"humor":False,"styles":[],"stylesHero":[],"stylesLogo":[],"stylesIcon":[],"imageMotionTypes":["static"],"sizes":[],"sizesHero":[],"sizesLogo":[],"sizesIcon":[]}},
            "defaultImage":image,"localImages":image,"parserId":pid,"version":22
        })
    write(target, json.dumps(old, ensure_ascii=False, indent=2)+"\n")

try:
    if mode == "mgba":
        mgba()
    elif mode == "melonds":
        melonds()
    elif mode == "srm":
        srm()
except (OSError, ValueError, TypeError) as ex:
    warn(str(ex))
    if mode == "srm":
        sys.exit(1)
