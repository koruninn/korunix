"""Recuperar los complementos y temas que faltan, sin borrar los actuales."""
import hashlib
import json
import os
import pwd
import re
import shutil
import stat
import sys
import tempfile
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath
from urllib.parse import quote, urlsplit

catalogo = json.loads(Path(sys.argv[1]).read_text())
base = Path(sys.argv[2])
usuario = pwd.getpwnam(sys.argv[3])
almacen = base / "plugins"
temas = base / "themes"
tienda = "https://plugins.deckbrew.xyz/plugins"
tienda_pruebas = "https://testing.deckbrew.xyz/plugins"
api_temas = "https://api.deckthemes.com"
limite = 256 * 1024 * 1024

def leer(ruta):
    return json.loads(ruta.read_text(encoding="utf-8"))

def solicitar(url):
    if urlsplit(url).scheme != "https":
        raise ValueError("Se requiere una dirección HTTPS")
    req = urllib.request.Request(url, headers={"User-Agent": "Korunix", "X-Decky-Version": "3.3.1"})
    respuesta = urllib.request.urlopen(req, timeout=25)
    if urlsplit(respuesta.geturl()).scheme != "https":
        respuesta.close()
        raise ValueError("Redirección fuera de HTTPS")
    return respuesta

def datos(url):
    with solicitar(url) as respuesta:
        contenido = respuesta.read(16 * 1024 * 1024 + 1)
    if len(contenido) > 16 * 1024 * 1024:
        raise ValueError("Respuesta demasiado grande")
    return json.loads(contenido)

def descargar(url, destino, suma=None):
    if suma is not None and not re.fullmatch(r"[0-9a-fA-F]{64}", suma):
        raise ValueError("SHA-256 inválido")
    hash_archivo = hashlib.sha256()
    total = 0
    with solicitar(url) as respuesta, destino.open("xb") as salida:
        while bloque := respuesta.read(1024 * 1024):
            total += len(bloque)
            if total > limite:
                raise ValueError("Descarga demasiado grande")
            hash_archivo.update(bloque)
            salida.write(bloque)
    if suma and hash_archivo.hexdigest() != suma.lower():
        raise ValueError("La descarga no coincide con la suma de la tienda")

def carpeta_segura(ruta):
    ruta.mkdir(exist_ok=True)
    if ruta.is_symlink() or not ruta.is_dir():
        raise ValueError(f"Ruta no válida: {ruta}")

def extraer(archivo, destino):
    with zipfile.ZipFile(archivo) as zip_file:
        if sum(i.file_size for i in zip_file.infolist()) > limite:
            raise ValueError("Archivo descomprimido demasiado grande")
        for info in zip_file.infolist():
            ruta = PurePosixPath(info.filename)
            modo = info.external_attr >> 16
            if (ruta.is_absolute() or ".." in ruta.parts or "\\" in info.filename
                    or stat.S_IFMT(modo) not in (0, stat.S_IFREG, stat.S_IFDIR)):
                raise ValueError("Ruta no válida dentro del ZIP")
            zip_file.extract(info, destino)

def encontrar(raiz, manifiesto, verificar):
    candidatos = [p.parent for p in raiz.rglob(manifiesto) if verificar(leer(p))]
    if len(candidatos) != 1:
        raise ValueError(f"No se encontró un único {manifiesto} válido")
    return candidatos[0]

def permisos(ruta, root=False):
    uid, gid = (0, 0) if root else (usuario.pw_uid, usuario.pw_gid)
    for elemento in [ruta, *ruta.rglob("*")]:
        if elemento.is_symlink():
            raise ValueError("No se admiten enlaces en los paquetes")
        os.chown(elemento, uid, gid)
        os.chmod(elemento, 0o755 if elemento.is_dir() else 0o644)

def instalado(raiz, manifiesto, verificar):
    for ruta in raiz.iterdir():
        if ruta.is_symlink() or not ruta.is_dir():
            continue
        archivo = ruta / manifiesto
        if archivo.is_file() and not archivo.is_symlink():
            try:
                if verificar(leer(archivo)):
                    return ruta
            except (OSError, ValueError):
                pass
    return None

def guardar_ajustes(ruta, ajustes):
    archivo = ruta / "config_USER.json"
    if not archivo.exists() and not archivo.is_symlink():
        with archivo.open("x", encoding="utf-8") as salida:
            json.dump(ajustes, salida, ensure_ascii=False, indent=2)
        os.chown(archivo, usuario.pw_uid, usuario.pw_gid)

def instalar_plugin(item, version):
    destino = almacen / item["carpeta"]
    if destino.exists() or destino.is_symlink():
        raise ValueError("Se conserva la carpeta existente")
    with tempfile.TemporaryDirectory(prefix=".decky-", dir=almacen) as tmp:
        tmp = Path(tmp)
        suma = version["hash"]
        url = version.get("artifact") or f"https://cdn.tzatzikiweeb.moe/file/steam-deck-homebrew/versions/{suma}.zip"
        descargar(url, tmp / "plugin.zip", suma)
        extraer(tmp / "plugin.zip", tmp / "contenido")
        origen = encontrar(tmp / "contenido", "plugin.json", lambda d: d.get("name") == item["nombre"] and d.get("author") == item["autor"])
        paquete = leer(origen / "package.json")
        if paquete.get("version") != item["version"]:
            raise ValueError("La versión del paquete no coincide")
        for binario in paquete.get("remote_binary", []):
            nombre = binario["name"]
            if nombre != Path(nombre).name or nombre in ("", ".", ".."):
                raise ValueError("Nombre de binario no válido")
            (origen / "bin").mkdir(exist_ok=True)
            descargar(binario["url"], origen / "bin" / nombre, binario["sha256hash"])
        permisos(origen, root="root" in leer(origen / "plugin.json").get("flags", []))
        # Decky necesita ejecutar los binarios que vienen con algunos complementos.
        for archivo in origen.rglob("*"):
            if archivo.is_file():
                archivo.chmod(0o755)
        os.chown(origen, 0, 0)
        os.chown(origen / "plugin.json", 0, 0)
        origen.rename(destino)

def instalar_tema(item, vistos):
    identificador = item["id"]
    if identificador in vistos:
        return
    vistos.add(identificador)
    actual = instalado(temas, "theme.json", lambda d: d.get("id") == identificador)
    if actual:
        if "ajustes" in item:
            guardar_ajustes(actual, item["ajustes"])
        return
    info = datos(f"{api_temas}/themes/{quote(identificador, safe='')}")
    for dep in info.get("dependencies", []):
        instalar_tema({"id": dep["id"], "carpeta": dep["name"]}, vistos)
    destino = temas / item["carpeta"]
    if destino.name != item["carpeta"] or destino.exists() or destino.is_symlink():
        raise ValueError("Se conserva la carpeta existente")
    with tempfile.TemporaryDirectory(prefix=".tema-", dir=temas) as tmp:
        tmp = Path(tmp)
        descargar(f"{api_temas}/blobs/{quote(info['download']['id'], safe='')}", tmp / "tema.zip")
        extraer(tmp / "tema.zip", tmp / "contenido")
        origen = encontrar(tmp / "contenido", "theme.json", lambda d: d.get("id") == identificador)
        version = leer(origen / "theme.json").get("version")
        if item.get("version") and version != item["version"]:
            print(f"{item['carpeta']}: la tienda ofrece {version}, no {item['version']}", flush=True)
        permisos(origen)
        if "ajustes" in item:
            guardar_ajustes(origen, item["ajustes"])
        origen.rename(destino)

def main():
    carpeta_segura(almacen)
    carpeta_segura(temas)
    errores = []
    faltantes = [p for p in catalogo["plugins"] if not instalado(almacen, "plugin.json", lambda d: d.get("name") == p["nombre"])]
    listas = {}
    for item in faltantes:
        try:
            elegida = None
            for url in (tienda, tienda_pruebas):
                if url not in listas:
                    listas[url] = datos(url)
                for plugin in listas[url]:
                    if plugin["name"] == item["nombre"] and plugin["author"] == item["autor"]:
                        elegida = next((v for v in plugin["versions"] if v["name"] == item["version"]), None)
                        if elegida:
                            break
                if elegida:
                    break
            if not elegida:
                raise ValueError("La versión declarada no está disponible en la tienda")
            instalar_plugin(item, elegida)
            print(f"Instalado: {item['nombre']}", flush=True)
        except Exception as error:
            errores.append(f"{item['nombre']}: {error}")
    for item in catalogo["temas"]:
        try:
            instalar_tema(item, set())
        except Exception as error:
            errores.append(f"{item['carpeta']}: {error}")
    for error in errores:
        print(error, file=sys.stderr)
    return 1 if errores else 0

if __name__ == "__main__":
    sys.exit(main())
