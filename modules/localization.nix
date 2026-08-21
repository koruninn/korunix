{
  config,
  lib,
  ...
}: let
  cfg = config.korunix.localization;

  # La primera distribución es la principal. Las adicionales permiten alternar
  # idiomas o variantes sin obligar a cada escritorio a inventar su propio modelo.
  keyboardLayouts =
    [cfg.keyboard.layout]
    ++ cfg.keyboard.additionalLayouts;

  keyboardVariants =
    [cfg.keyboard.variant]
    ++ cfg.keyboard.additionalVariants;
in {
  options.korunix.localization = {
    language = lib.mkOption {
      type = lib.types.str;
      default = "es";
      description = "Idioma preferido de la interfaz y del sistema.";
    };

    region = lib.mkOption {
      type = lib.types.str;
      default = "PE";
      description = "Región utilizada para formatos como fechas y números.";
    };

    timeZone = lib.mkOption {
      type = lib.types.str;
      default = "America/Lima";
      description = "Zona horaria del equipo.";
    };

    keyboard = {
      layout = lib.mkOption {
        type = lib.types.str;
        default = "es";
        description = "Distribución principal del teclado.";
      };

      variant = lib.mkOption {
        type = lib.types.str;
        default = "";
        description = "Variante de la distribución principal cuando sea necesaria.";
      };

      additionalLayouts = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Distribuciones adicionales entre las que la persona puede alternar.";
      };

      additionalVariants = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Variantes correspondientes a las distribuciones adicionales.";
      };

      displayNames = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Nombres humanos de las distribuciones, en el mismo orden que los layouts.";
      };

      switchOption = lib.mkOption {
        type = lib.types.str;
        default = "grp:alt_shift_toggle";
        description = "Combinación XKB utilizada para cambiar de distribución.";
      };

      console = lib.mkOption {
        type = lib.types.str;
        default = "es";
        description = "Mapa de teclado utilizado fuera de la sesión gráfica.";
      };
    };
  };

  config = lib.mkIf config.korunix.enable {
    assertions = [
      {
        assertion =
          builtins.length cfg.keyboard.additionalLayouts
          == builtins.length cfg.keyboard.additionalVariants;

        message =
          "Cada distribución adicional de teclado necesita su variante correspondiente, "
          + "aunque esa variante sea una cadena vacía.";
      }
      {
        assertion =
          cfg.keyboard.displayNames == []
          || builtins.length cfg.keyboard.displayNames
          == builtins.length keyboardLayouts;

        message =
          "Los nombres humanos del teclado deben corresponder uno a uno con las distribuciones.";
      }
    ];

    # Idioma y región son decisiones independientes.
    i18n.defaultLocale = "${cfg.language}_${cfg.region}.UTF-8";

    time.timeZone = cfg.timeZone;

    # systemd-localed publica esta configuración a los escritorios. Niri puede
    # consumirla directamente y otros escritorios obtienen el mismo orden.
    services.xserver.xkb = {
      layout = lib.concatStringsSep "," keyboardLayouts;
      variant = lib.concatStringsSep "," keyboardVariants;
      options = cfg.keyboard.switchOption;
    };

    console.keyMap = cfg.keyboard.console;
  };
}
