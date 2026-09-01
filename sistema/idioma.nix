{
  config,
  lib,
  ...
}: let
  cfg = config.korunix.localization;

  # El idioma del sistema y la región son decisiones distintas. Por ejemplo,
  # una persona puede querer interfaces en español y formatos de otro país.
  localeFor = language: region: "${language}_${region}.UTF-8";

  systemLocale = localeFor cfg.systemLanguage cfg.region;
  formatLocale = localeFor cfg.formats.language cfg.formats.region;

  # LANGUAGE conserva un orden de preferencias para aplicaciones que pueden
  # ofrecer más de un idioma sin alterar el locale base ni los formatos.
  preferredLanguages =
    if cfg.preferredLanguages == null
    then [cfg.systemLanguage]
    else cfg.preferredLanguages;

  # La primera distribución es la principal. Las adicionales mantienen el mismo
  # orden en NixOS, Niri, Hyprland y las etiquetas humanas de Noctalia.
  keyboardLayouts =
    [cfg.keyboard.layout]
    ++ cfg.keyboard.additionalLayouts;

  keyboardVariants =
    [cfg.keyboard.variant]
    ++ cfg.keyboard.additionalVariants;

  validLanguage = language:
    builtins.match "^[a-z][a-z][a-z]?$" language != null;

  validRegion = region:
    builtins.match "^[A-Z][A-Z]$" region != null;
in {
  options.korunix.localization = {
    systemLanguage = lib.mkOption {
      type = lib.types.str;
      default = "es";
      description = ''
        Idioma base del sistema. No representa la preferencia portable de una
        persona concreta.
      '';
    };

    preferredLanguages = lib.mkOption {
      type = lib.types.nullOr (lib.types.listOf lib.types.str);
      default = null;
      description = ''
        Idiomas preferidos del sistema en orden. El primero coincide con
        systemLanguage. null conserva compatibilidad y equivale a usar solo el
        idioma base.
      '';
    };

    region = lib.mkOption {
      type = lib.types.str;
      default = "PE";
      description = ''
        Región base de este equipo. Forma parte de su contexto local y no viaja
        dentro de un perfil portable.
      '';
    };

    formats = {
      language = lib.mkOption {
        type = lib.types.str;
        default = "es";
        description = ''
          Idioma utilizado al construir el locale de formatos regionales.
        '';
      };

      region = lib.mkOption {
        type = lib.types.str;
        default = "PE";
        description = ''
          Región usada para fechas, números, moneda, medidas, papel, direcciones
          y otros formatos que no necesitan controlar el idioma de la interfaz.
        '';
      };
    };

    timeZone = lib.mkOption {
      type = lib.types.str;
      default = "America/Lima";
      description = ''
        Zona horaria de este equipo. Es estado local del host y no una preferencia
        portable del usuario.
      '';
    };

    keyboard = {
      layout = lib.mkOption {
        type = lib.types.str;
        default = "es";
        description = "Distribución principal del teclado de este equipo.";
      };

      variant = lib.mkOption {
        type = lib.types.str;
        default = "";
        description = ''
          Variante de la distribución principal cuando sea necesaria.
        '';
      };

      additionalLayouts = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = ''
          Distribuciones adicionales disponibles para alternar en este equipo.
        '';
      };

      additionalVariants = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = ''
          Variantes correspondientes a las distribuciones adicionales.
        '';
      };

      displayNames = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = ''
          Nombres humanos de los teclados, en el mismo orden que los layouts.
        '';
      };

      switchOption = lib.mkOption {
        type = lib.types.str;
        default = "grp:alt_shift_toggle";
        description = ''
          Combinación XKB para cambiar de distribución. Super+Espacio permanece
          reservado al launcher común de Korunix.
        '';
      };

      console = lib.mkOption {
        type = lib.types.str;
        default = "es";
        description = ''
          Mapa de teclado utilizado fuera de la sesión gráfica.
        '';
      };
    };
  };

  config = lib.mkIf config.korunix.enable {
    assertions = [
      {
        assertion = validLanguage cfg.systemLanguage;
        message =
          "korunix.localization.systemLanguage debe ser un código de idioma "
          + "compatible con locale, por ejemplo es, en o pt.";
      }
      {
        assertion =
          preferredLanguages
          != []
          && builtins.all validLanguage preferredLanguages
          && builtins.head preferredLanguages == cfg.systemLanguage
          && builtins.length preferredLanguages
          == builtins.length (lib.unique preferredLanguages);
        message =
          "korunix.localization.preferredLanguages debe contener idiomas "
          + "válidos, sin repetir, y comenzar por systemLanguage.";
      }
      {
        assertion = validRegion cfg.region;
        message =
          "korunix.localization.region debe ser un código regional de dos "
          + "letras mayúsculas, por ejemplo PE.";
      }
      {
        assertion = validLanguage cfg.formats.language;
        message =
          "korunix.localization.formats.language debe ser un código de idioma "
          + "compatible con locale.";
      }
      {
        assertion = validRegion cfg.formats.region;
        message =
          "korunix.localization.formats.region debe ser un código regional de "
          + "dos letras mayúsculas.";
      }
      {
        assertion =
          builtins.length cfg.keyboard.additionalLayouts
          == builtins.length cfg.keyboard.additionalVariants;

        message =
          "Cada distribución adicional de teclado necesita su variante "
          + "correspondiente, aunque sea una cadena vacía.";
      }
      {
        assertion =
          cfg.keyboard.displayNames
          == []
          || builtins.length cfg.keyboard.displayNames
          == builtins.length keyboardLayouts;

        message =
          "Los nombres humanos del teclado deben corresponder uno a uno con "
          + "las distribuciones.";
      }
      {
        assertion = !(lib.hasInfix "win_space" cfg.keyboard.switchOption);
        message =
          "Super+Espacio está reservado al launcher de Korunix. El cambio de "
          + "teclado no puede utilizar una opción XKB win_space.";
      }
    ];

    # LANG conserva el idioma base del sistema. Los formatos regionales pueden
    # evolucionar de forma independiente sin alterar los mensajes de interfaz.
    i18n.defaultLocale = systemLocale;

    environment.sessionVariables.LANGUAGE =
      lib.concatStringsSep ":" preferredLanguages;

    i18n.extraLocaleSettings = {
      LC_ADDRESS = formatLocale;
      LC_IDENTIFICATION = formatLocale;
      LC_MEASUREMENT = formatLocale;
      LC_MONETARY = formatLocale;
      LC_NAME = formatLocale;
      LC_NUMERIC = formatLocale;
      LC_PAPER = formatLocale;
      LC_TELEPHONE = formatLocale;
      LC_TIME = formatLocale;
    };

    time.timeZone = cfg.timeZone;

    services.xserver.xkb = {
      layout = lib.concatStringsSep "," keyboardLayouts;
      variant = lib.concatStringsSep "," keyboardVariants;
      options = cfg.keyboard.switchOption;
    };

    console.keyMap = cfg.keyboard.console;
  };
}
