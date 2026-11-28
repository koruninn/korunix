import os
from pathlib import Path
import tempfile
import yaml

path = Path(os.environ.get("HHD_CONFIG_DIR", "/etc/hhd")) / "state.yml"
path.parent.mkdir(parents=True, exist_ok=True)
legacy = Path("/home/koru/.config/hhd/state.yml")
source = path if path.exists() else legacy
state = yaml.safe_load(source.read_text()) if source.is_file() else {}
if state is None:
    state = {}
if not isinstance(state, dict):
    raise ValueError("La configuración de HHD no es válida.")
hhd = state.setdefault("hhd", {})
if not isinstance(hhd, dict):
    raise ValueError("La sección hhd no es válida.")
settings = hhd.setdefault("settings", {})
if not isinstance(settings, dict):
    raise ValueError("Los ajustes de HHD no son válidos.")
if path.exists() and settings.get("powerbuttond") is False:
    return_value = False
else:
    settings["powerbuttond"] = False
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as f:
        yaml.safe_dump(state, f, sort_keys=False)
        name = f.name
    os.chmod(name, 0o600)
    os.replace(name, path)
