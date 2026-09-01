# Unidades de datos que Korunix deja listas para el uso cotidiano.
#
# La interfaz habla de disponibilidad, no de montaje. Internamente se usa el
# UUID del sistema de archivos porque /dev/sda, /dev/sdb, etc. pueden cambiar
# entre arranques. x-systemd.automount deja la ruta preparada desde el arranque
# y realiza el acceso real al primer uso sin pedir otra autorización.
{
  config,
  lib,
  ...
}: let
  cfg = config.korunix.storage;

  tipoEfectivo = volumen:
    if volumen.fileSystem == "ntfs"
    then "ntfs3"
    else volumen.fileSystem;

  necesitaPropietarioSintetico = volumen:
    lib.elem volumen.fileSystem [
      "ntfs"
      "exfat"
      "vfat"
    ];

  opcionesPara = volumen:
    [
      "nofail"
      "x-systemd.automount"
      "x-systemd.device-timeout=5s"
    ]
    ++ lib.optionals (necesitaPropietarioSintetico volumen) [
      "uid=${toString volumen.ownerUid}"
      "gid=${toString volumen.ownerGid}"
      "umask=0077"
    ]
    ++ lib.optionals (volumen.fileSystem == "ntfs") [
      "windows_names"
    ];

  activas = lib.filter (volumen: volumen.availableAtLogin) cfg.dataVolumes;

  ids = map (volumen: volumen.id) cfg.dataVolumes;
  uuids = map (volumen: lib.toLower volumen.uuid) cfg.dataVolumes;
  rutas = map (volumen: volumen.path) cfg.dataVolumes;
in {
  options.korunix.storage.dataVolumes = lib.mkOption {
    default = [];

    type = lib.types.listOf (lib.types.submodule {
      options = {
        id = lib.mkOption {
          type = lib.types.str;
          description = "Nombre interno estable de la unidad de datos.";
        };

        uuid = lib.mkOption {
          type = lib.types.str;
          description = "UUID detectado del sistema de archivos.";
        };

        fileSystem = lib.mkOption {
          type = lib.types.enum [
            "ntfs"
            "ext4"
            "btrfs"
            "exfat"
            "vfat"
          ];
          description = "Formato detectado de la unidad.";
        };

        path = lib.mkOption {
          type = lib.types.str;
          description = "Ruta estable que las aplicaciones pueden utilizar.";
        };

        # Estos números son hechos locales detectados por Korunix. La persona no
        # debe introducirlos manualmente desde la interfaz.
        ownerUid = lib.mkOption {
          type = lib.types.int;
          description = "UID local que recibe acceso cuando el formato no guarda permisos POSIX.";
        };

        ownerGid = lib.mkOption {
          type = lib.types.int;
          description = "GID local asociado al acceso de la unidad.";
        };

        availableAtLogin = lib.mkOption {
          type = lib.types.bool;
          default = true;
          description = "Deja la unidad lista automáticamente para las sesiones del equipo.";
        };
      };
    });

    description = "Unidades de datos administradas por Korunix.";
  };

  config = lib.mkIf config.korunix.enable {
    assertions = [
      {
        assertion = lib.length ids == lib.length (lib.unique ids);
        message = "Korunix no admite dos unidades de datos con el mismo identificador.";
      }
      {
        assertion = lib.length uuids == lib.length (lib.unique uuids);
        message = "Korunix no admite dos unidades de datos con el mismo UUID.";
      }
      {
        assertion = lib.length rutas == lib.length (lib.unique rutas);
        message = "Korunix no admite dos unidades de datos en la misma ruta.";
      }
      {
        assertion = lib.all (volumen: volumen.uuid != "") cfg.dataVolumes;
        message = "Toda unidad de datos necesita un UUID estable.";
      }
      {
        assertion = lib.all (volumen: lib.hasPrefix "/" volumen.path && volumen.path != "/") cfg.dataVolumes;
        message = "Las unidades de datos necesitan una ruta absoluta distinta de /.";
      }
      {
        assertion = lib.all (volumen: volumen.ownerUid >= 0 && volumen.ownerGid >= 0) cfg.dataVolumes;
        message = "Los identificadores locales de acceso de una unidad no pueden ser negativos.";
      }
    ];

    fileSystems = lib.listToAttrs (map (volumen: {
        name = volumen.path;
        value = {
          device = "/dev/disk/by-uuid/${volumen.uuid}";
          fsType = tipoEfectivo volumen;
          options = opcionesPara volumen;
        };
      })
      activas);
  };
}
