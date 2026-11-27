import json
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlsplit


def main():
    fuentes = json.loads(Path(sys.argv[1]).read_text())
    gearlever = sys.argv[2]
    home = Path.home()
    correcto = True
    for fuente in fuentes:
        patron = fuente["archivo"]
        url = fuente["url"]
        ruta = Path(patron)
        if ruta.is_absolute() or ".." in ruta.parts or urlsplit(url).scheme != "https":
            print("Gear Lever: declaración no válida: " + patron, file=sys.stderr)
            correcto = False
            continue

        encontrados = sorted(p for p in home.glob(patron)
                             if p.is_file() and not p.is_symlink()
                             and p.suffix.lower() == ".appimage")
        if len(encontrados) != 1:
            print("Gear Lever: se esperaba un único AppImage para " + patron,
                  file=sys.stderr)
            correcto = False
            continue

        comando = [gearlever, "--set-update-url", str(encontrados[0]), "--url", url]
        if fuente.get("gestor"):
            comando += ["--manager", fuente["gestor"]]
        if subprocess.run(comando).returncode != 0:
            correcto = False

    return 0 if correcto else 1


if __name__ == "__main__":
    sys.exit(main())
