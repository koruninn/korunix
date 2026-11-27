import json
import os
import pwd
import shutil
import subprocess
import sys
from pathlib import Path

ETIQUETA = "NixDeck"


def ejecutar(*args):
    return subprocess.check_output(args, text=True).strip()


def discos():
    return json.loads(ejecutar(
        "lsblk", "--json", "--list", "--paths",
        "--output", "PATH,TYPE,FSTYPE,LABEL,UUID,MAJ:MIN,PKNAME,RM,SIZE,MOUNTPOINTS"
    ))["blockdevices"]


def montajes(disco):
    return [p for p in disco.get("mountpoints") or [] if p]


def extraible(disco, dispositivos):
    padre = dispositivos.get(disco.get("pkname"), {})
    if disco.get("type") != "part" or padre.get("type") != "disk":
        return False
    nombre = Path(padre["path"]).name
    tipo = Path("/sys/class/block") / nombre / "device/type"
    if nombre.startswith("mmcblk"):
        try:
            es_sd = tipo.read_text().strip() == "SD"
        except OSError:
            es_sd = False
        if not es_sd:
            return False
    elif not (disco.get("rm") or padre.get("rm")):
        return False

    # No modificar el disco desde el que arrancó NixOS.
    for otro in dispositivos.values():
        actual = otro
        visitados = set()
        while actual and actual["path"] not in visitados:
            visitados.add(actual["path"])
            if actual["path"] == padre["path"]:
                if any(not p.startswith(("/run/media/", "/media/", "/mnt/"))
                       for p in montajes(otro)):
                    return False
                break
            actual = dispositivos.get(actual.get("pkname"))
    return True


def etiquetados(lista):
    return [d for d in lista if d.get("label") == ETIQUETA]


def unica(lista):
    tarjetas = etiquetados(lista)
    if len(tarjetas) != 1 or tarjetas[0].get("fstype") != "ext4":
        raise ValueError("Conecta una sola partición ext4 con la etiqueta NixDeck.")
    tarjeta = tarjetas[0]
    enlace = Path("/dev/disk/by-label") / ETIQUETA
    if not enlace.exists() or enlace.resolve() != Path(tarjeta["path"]).resolve():
        raise ValueError("La etiqueta NixDeck todavía no está disponible.")
    return tarjeta


def comprobar(montaje):
    tarjeta = unica(discos())
    os.listdir(montaje)
    actual = unica(discos())
    if actual["maj:min"] != tarjeta["maj:min"]:
        raise ValueError("La tarjeta cambió durante el montaje.")
    salida = ejecutar("findmnt", "-rn", "-o", "MAJ:MIN,FSTYPE",
                      "--mountpoint", montaje)
    reales = [linea.split() for linea in salida.splitlines()
              if linea.split() and linea.split()[-1] != "autofs"]
    if reales != [[actual["maj:min"], "ext4"]]:
        raise ValueError("La tarjeta NixDeck no está montada en " + montaje + ".")
    return actual


def configurar(montaje, usuario):
    if os.getuid() == 0 or pwd.getpwuid(os.getuid()).pw_name != usuario:
        raise ValueError("Ejecuta este comando desde tu cuenta habitual, sin sudo.")
    lista = discos()
    dispositivos = {d["path"]: d for d in lista}
    candidatos = [d for d in lista if d.get("fstype") == "ext4"
                  and extraible(d, dispositivos)]
    if not candidatos:
        raise ValueError("No hay particiones ext4 en tarjetas o discos extraíbles. "
                         "Comprueba la tarjeta con lsblk -f.")
    for i, d in enumerate(candidatos, 1):
        print(f'{i}. {d["path"]} — {d["size"]} — {d.get("label") or "sin etiqueta"}'
              f' — {", ".join(montajes(d)) or "sin montar"}')
    respuesta = input("Selecciona la partición (Enter para cancelar): ").strip()
    if not respuesta:
        return False
    if not respuesta.isdigit() or not 1 <= int(respuesta) <= len(candidatos):
        raise ValueError("Selección no válida.")
    elegida = candidatos[int(respuesta) - 1]
    ruta = elegida["path"]
    if any(d["path"] != ruta for d in etiquetados(lista)):
        raise ValueError("Ya hay otra partición con la etiqueta NixDeck. Retírala primero.")
    print(f'Se preparará {ruta} ({elegida["size"]}). No se formateará.')
    if input("Escribe NixDeck para confirmar: ").strip() != ETIQUETA:
        return False

    lista = discos()
    dispositivos = {d["path"]: d for d in lista}
    actual = dispositivos.get(ruta)
    if (not actual or actual.get("fstype") != "ext4"
            or not extraible(actual, dispositivos)
            or any(actual.get(k) != elegida.get(k)
                   for k in ("uuid", "maj:min", "pkname", "label", "size"))):
        raise ValueError("La partición cambió. Vuelve a ejecutar el comando.")
    if any(d["path"] != ruta for d in etiquetados(lista)):
        raise ValueError("Hay otra partición con la etiqueta NixDeck.")

    if actual.get("label") != ETIQUETA:
        if montajes(actual):
            raise ValueError("Desmonta primero la partición desde el explorador "
                             "de archivos o Discos y vuelve a intentarlo. No la formatees.")
        subprocess.run(["sudo", shutil.which("e2label"), ruta, ETIQUETA], check=True)
        subprocess.run(["sudo", shutil.which("udevadm"), "trigger", "--action=change",
                        "--settle", ruta], check=True)

    tarjeta = unica(discos())
    if tarjeta["maj:min"] != elegida["maj:min"] or tarjeta.get("uuid") != elegida.get("uuid"):
        raise ValueError("No se pudo verificar la tarjeta seleccionada.")
    if any(p != montaje for p in montajes(tarjeta)):
        raise ValueError("Desmonta la tarjeta de su ubicación actual y vuelve a intentarlo.")
    comprobar(montaje)
    if not os.access(montaje, os.W_OK):
        subprocess.run(["sudo", shutil.which("chown"), "--no-dereference",
                        usuario, montaje], check=True)
    comprobar(montaje)
    if not os.access(montaje, os.W_OK):
        raise ValueError("No tienes permiso para escribir en la tarjeta.")
    return True


def main():
    modo, montaje, usuario = sys.argv[1:]
    try:
        if modo == "comprobar":
            comprobar(montaje)
        elif modo == "configurar":
            if not configurar(montaje, usuario):
                print("Cancelado.")
                return 2
        else:
            raise ValueError("Modo desconocido.")
    except (ValueError, OSError, subprocess.CalledProcessError, EOFError, KeyboardInterrupt) as error:
        print("NixDeck: " + (str(error) or "operación cancelada"), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
