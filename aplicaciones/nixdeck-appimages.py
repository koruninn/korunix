import fcntl
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

def curl(url, destino=None):
    comando = ["curl", "--fail", "--location", "--silent", "--show-error",
               "--proto", "=https", "--proto-redir", "=https",
               "--connect-timeout", "15", "--max-time", "600", "--retry", "2"]
    if destino is not None:
        comando += ["--output", str(destino)]
    else:
        comando += ["--header", "Accept: application/vnd.github+json"]
    return subprocess.check_output(comando + [url])


def sha256(ruta):
    with ruta.open("rb") as archivo:
        return hashlib.file_digest(archivo, "sha256").hexdigest()


def imagen_valida(ruta):
    if ruta.is_symlink() or not ruta.is_file():
        return False
    with ruta.open("rb") as archivo:
        cabecera = archivo.read(12)
    return cabecera.startswith(b"\x7fELF") and cabecera[8:11] in (b"AI\x01", b"AI\x02")


def destino_actual(carpeta, nombre, archivo):
    if nombre == "Xenia":
        candidatos = sorted(p for p in carpeta.iterdir()
                            if "xenia" in p.name.lower()
                            and p.suffix.lower() == ".appimage")
        if len(candidatos) > 1:
            raise ValueError("Deja un solo AppImage de Xenia en appimages.")
        if candidatos:
            return candidatos[0]
    return carpeta / archivo


def instalar(carpeta, nombre, repo, version, archivo):
    destino = destino_actual(carpeta, nombre, archivo)
    if destino.is_symlink() or (destino.exists() and not destino.is_file()):
        raise ValueError("Ruta no compatible: " + str(destino))

    try:
        datos = json.loads(curl(f"https://api.github.com/repos/{repo}/releases/{version}"))
        recurso = next((a for a in datos["assets"] if a["name"] == archivo), None)
        if not recurso:
            raise ValueError("La publicación no contiene el AppImage esperado.")
        digest = recurso.get("digest") or ""
        if not re.fullmatch(r"sha256:[0-9a-fA-F]{64}", digest):
            raise ValueError("La publicación no proporciona un SHA-256 válido.")
        esperado = digest.split(":", 1)[1].lower()
        url = recurso["browser_download_url"]
        if not url.startswith(f"https://github.com/{repo}/releases/download/") or not url.endswith("/" + archivo):
            raise ValueError("Dirección de descarga inesperada.")

        if imagen_valida(destino) and sha256(destino) == esperado:
            destino.chmod(destino.stat().st_mode | 0o100)
            print(f"{nombre}: al día.")
            return True

        print(f"{nombre}: descargando {datos['tag_name']}...", flush=True)
        with tempfile.TemporaryDirectory(prefix=".nixdeck-", dir=carpeta) as temporal:
            nueva = Path(temporal) / archivo
            curl(url, nueva)
            if (not imagen_valida(nueva) or nueva.stat().st_size != recurso["size"]
                    or sha256(nueva) != esperado):
                raise ValueError("La descarga no coincide con la publicación.")
            nueva.chmod(0o755)
            if imagen_valida(destino):
                anteriores = carpeta / ".anteriores"
                anteriores.mkdir(exist_ok=True)
                if anteriores.is_symlink():
                    raise ValueError("La carpeta de respaldo no puede ser un enlace.")
                copia = Path(temporal) / "anterior"
                os.link(destino, copia)
                os.replace(copia, anteriores / archivo)
            os.replace(nueva, destino)
        print(f"{nombre}: listo.")
        return True
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        print(f"NixDeck: {nombre}: {error}", file=sys.stderr)
        if imagen_valida(destino):
            print(f"{nombre}: se conserva la versión instalada.", file=sys.stderr)
            return True
        return False


def main():
    carpeta = Path(sys.argv[1]) / "appimages"
    proyectos = json.loads(Path(sys.argv[2]).read_text())
    if carpeta.is_symlink() or not carpeta.is_dir():
        raise ValueError("No se encuentra la carpeta appimages de NixDeck.")
    with (carpeta / ".nixdeck-descargas.lock").open("a") as bloqueo:
        fcntl.flock(bloqueo, fcntl.LOCK_EX)
        correcto = True
        for proyecto in proyectos:
            try:
                correcto = instalar(carpeta, **proyecto) and correcto
            except (OSError, ValueError) as error:
                print(f"NixDeck: {proyecto['nombre']}: {error}", file=sys.stderr)
                correcto = False
    return 0 if correcto else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError) as error:
        print("NixDeck: " + str(error), file=sys.stderr)
        sys.exit(1)
