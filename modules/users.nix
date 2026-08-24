{
  config,
  korunixContext,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix;

  # Cada usuario es un archivo de datos legible. Si el host menciona un ID que no
  # existe, detener la evaluación es más seguro que crear una cuenta incompleta.
  loadUser = userId: let
    path = korunixContext.usersPath + "/${userId}.nix";
  in
    if builtins.pathExists path
    then import path
    else throw "El host ${cfg.hostId} utiliza ${userId}, pero falta users/${userId}.nix.";

  profiles = lib.genAttrs cfg.users loadUser;

  settingsFor = userId: cfg.userSettings.${userId} or {};

  accountNameFor = userId: let
    profile = profiles.${userId};
    settings = settingsFor userId;
    localName = settings.accountName or null;
  in
    if localName != null
    then localName
    else profile.accountName or userId;

  homeDirectoryFor = userId: let
    settings = settingsFor userId;
    localHome = settings.homeDirectory or null;
  in
    if localHome != null
    then localHome
    else "/home/${accountNameFor userId}";

  administratorFor = userId:
    (settingsFor userId).administrator or false;

  requestedCapabilitiesFor = userId:
    profiles.${userId}.capabilities or [];

  deferredCapabilitiesFor = userId:
    (settingsFor userId).deferredCapabilities or [];

  enabledCapabilitiesFor = userId:
    lib.filter
    (
      capability:
        !(lib.elem capability (deferredCapabilitiesFor userId))
    )
    (requestedCapabilitiesFor userId);

  requestedInputMethodsFor = userId:
    profiles.${userId}.inputMethods or [];

  deferredInputMethodsFor = userId:
    (settingsFor userId).deferredInputMethods or [];

  enabledInputMethodsFor = userId:
    lib.filter
    (
      inputMethod:
        !(lib.elem inputMethod (deferredInputMethodsFor userId))
    )
    (requestedInputMethodsFor userId);

  # El perfil portable solo conoce decisiones humanas. La traducción a motores
  # y paquetes concretos pertenece a este módulo del host.
  inputMethodCatalog = [
    {
      id = "chinese-pinyin";
      label = "Chino — Pinyin";
      engine = "pinyin";
    }
    {
      id = "korean-hangul";
      label = "Coreano — Hangul";
      engine = "hangul";
    }
    {
      id = "japanese-mozc";
      label = "Japonés — Mozc";
      engine = "mozc";
    }
    {
      id = "vietnamese-unikey";
      label = "Vietnamita — Unikey";
      engine = "unikey";
    }
  ];

  knownInputMethods = map (entry: entry.id) inputMethodCatalog;

  inputMethodEntryFor = inputMethod:
    lib.findFirst
    (entry: entry.id == inputMethod)
    (throw "Korunix no conoce el método de entrada ${inputMethod}.")
    inputMethodCatalog;

  fcitxEngineFor = inputMethod:
    (inputMethodEntryFor inputMethod).engine;

  fcitxAddonFor = inputMethod:
    if inputMethod == "chinese-pinyin"
    then pkgs.qt6Packages.fcitx5-chinese-addons
    else if inputMethod == "korean-hangul"
    then pkgs.fcitx5-hangul
    else if inputMethod == "japanese-mozc"
    then pkgs.fcitx5-mozc
    else if inputMethod == "vietnamese-unikey"
    then pkgs.qt6Packages.fcitx5-unikey
    else throw "Korunix no tiene un addon para ${inputMethod}.";

  # Filtrar primero permite que una preferencia desconocida llegue a una
  # assertion humana en vez de provocar un error opaco al resolver paquetes.
  usableInputMethodsFor = userId:
    lib.filter
    (inputMethod: lib.elem inputMethod knownInputMethods)
    (enabledInputMethodsFor userId);

  hostInputMethods = lib.unique (
    lib.concatMap usableInputMethodsFor cfg.users
  );

  advancedInputMethodsEnabled = hostInputMethods != [];

  fcitxAddons = map fcitxAddonFor hostInputMethods;

  # Fcitx5 usa layout-variante para identificar dinámicamente un teclado XKB.
  fcitxDefaultLayout =
    if config.korunix.localization.keyboard.variant == ""
    then config.korunix.localization.keyboard.layout
    else
      config.korunix.localization.keyboard.layout
      + "-"
      + config.korunix.localization.keyboard.variant;

  fcitxKeyboard = "keyboard-" + fcitxDefaultLayout;

  preservedGroupsFor = userId:
    (settingsFor userId).preservedGroups or [];

  githubSshIdentityFileFor = userId:
    (settingsFor userId).githubSshIdentityFile or null;

  githubSshConfig = lib.concatStringsSep "\n" (
    lib.filter
    (entry: entry != "")
    (map (
        userId: let
          identity = githubSshIdentityFileFor userId;
          accountName = accountNameFor userId;
          homeDirectory = homeDirectoryFor userId;
          identityPath =
            if identity == null
            then null
            else if lib.hasPrefix "/" identity
            then identity
            else "${homeDirectory}/${identity}";
        in
          lib.optionalString (identity != null) ''
            Match host github.com localuser ${accountName}
              IdentityFile ${identityPath}
              IdentitiesOnly yes
              AddKeysToAgent yes
          ''
      )
      cfg.users)
  );

  # La interfaz trabaja con capacidades humanas. Esta lista impide aceptar un
  # nombre inventado que luego no tenga una traducción real en el sistema.
  knownCapabilities = [
    "android"
    "printing"
    "sunshine"
    "virtualization"
  ];

  settingsIds = builtins.attrNames cfg.userSettings;

  missingSettingsIds =
    lib.filter
    (userId: !(builtins.hasAttr userId cfg.userSettings))
    cfg.users;

  extraSettingsIds =
    lib.filter
    (userId: !(lib.elem userId cfg.users))
    settingsIds;

  accountNames = map accountNameFor cfg.users;

  administratorIds =
    lib.filter
    administratorFor
    cfg.users;

  unknownCapabilities =
    lib.concatMap
    (
      userId:
        map
        (capability: "${userId}:${capability}")
        (
          lib.filter
          (capability: !(lib.elem capability knownCapabilities))
          (requestedCapabilitiesFor userId)
        )
    )
    cfg.users;

  unknownDeferredCapabilities =
    lib.concatMap
    (
      userId:
        map
        (capability: "${userId}:${capability}")
        (
          lib.filter
          (capability: !(lib.elem capability knownCapabilities))
          (deferredCapabilitiesFor userId)
        )
    )
    cfg.users;

  invalidDeferredCapabilities =
    lib.concatMap
    (
      userId:
        map
        (capability: "${userId}:${capability}")
        (
          lib.filter
          (
            capability:
              !(lib.elem capability (requestedCapabilitiesFor userId))
          )
          (deferredCapabilitiesFor userId)
        )
    )
    cfg.users;

  unknownInputMethods =
    lib.concatMap
    (
      userId:
        map
        (inputMethod: "${userId}:${inputMethod}")
        (
          lib.filter
          (inputMethod: !(lib.elem inputMethod knownInputMethods))
          (requestedInputMethodsFor userId)
        )
    )
    cfg.users;

  unknownDeferredInputMethods =
    lib.concatMap
    (
      userId:
        map
        (inputMethod: "${userId}:${inputMethod}")
        (
          lib.filter
          (inputMethod: !(lib.elem inputMethod knownInputMethods))
          (deferredInputMethodsFor userId)
        )
    )
    cfg.users;

  invalidDeferredInputMethods =
    lib.concatMap
    (
      userId:
        map
        (inputMethod: "${userId}:${inputMethod}")
        (
          lib.filter
          (
            inputMethod:
              !(lib.elem inputMethod (requestedInputMethodsFor userId))
          )
          (deferredInputMethodsFor userId)
        )
    )
    cfg.users;

  # En systemd 258 Android ya no necesita adbusers. Si está activa en este host,
  # android-tools es la implementación que debe existir.
  androidRequested =
    lib.any
    (
      userId:
        lib.elem "android" (enabledCapabilitiesFor userId)
    )
    cfg.users;

  groupsFor = userId: let
    capabilities = enabledCapabilitiesFor userId;
  in
    lib.unique (
      ["networkmanager"]
      ++ lib.optionals (administratorFor userId) ["wheel"]
      ++ lib.optionals (lib.elem "virtualization" capabilities) [
        "libvirtd"
        "kvm"
      ]
      ++ lib.optionals (lib.elem "sunshine" capabilities) [
        "input"
        "uinput"
      ]
      ++ lib.optionals (lib.elem "printing" capabilities) [
        "lp"
        "scanner"
      ]
      # Los grupos conservados son estado de adopción de este host y nunca viajan
      # dentro de un perfil portable.
      ++ (preservedGroupsFor userId)
    );

  usersConfig = lib.listToAttrs (map (
      userId: let
        profile = profiles.${userId};
        accountName = accountNameFor userId;
      in {
        name = accountName;
        value = {
          isNormalUser = true;
          description = profile.fullName;
          home = homeDirectoryFor userId;
          shell = pkgs.fish;
          extraGroups = groupsFor userId;
        };
      }
    )
    cfg.users);

  # El mismo servicio existe en cada sesión, pero solo continúa si la cuenta
  # actual pertenece al host. Esto permite varios usuarios sin duplicar módulos.
  accountCases = lib.concatStringsSep "\n" (map (
      userId: let
        profile = profiles.${userId};
        accountName = accountNameFor userId;
        avatar = profile.avatar or null;
        language = profile.language or config.korunix.localization.systemLanguage;
        inputMethods = usableInputMethodsFor userId;
        fcitxMethods = map fcitxEngineFor inputMethods;
      in ''
        ${lib.escapeShellArg accountName})
          KORUNIX_USER_ID=${lib.escapeShellArg userId}
          KORUNIX_LANGUAGE=${lib.escapeShellArg language}
          KORUNIX_INPUT_METHODS=${lib.escapeShellArg (lib.concatStringsSep "," inputMethods)}
          KORUNIX_FCITX_METHODS=${lib.escapeShellArg (lib.concatStringsSep "," fcitxMethods)}
          KORUNIX_FCITX_ENABLED=${
          if advancedInputMethodsEnabled
          then "1"
          else "0"
        }
          KORUNIX_FCITX_KEYBOARD=${lib.escapeShellArg fcitxKeyboard}
          KORUNIX_AVATAR_SOURCE=${
          if avatar == null
          then "''"
          else lib.escapeShellArg (toString avatar)
        }
          ;;
      ''
    )
    cfg.users);

  prepareUser = pkgs.writeShellScript "korunix-user-prepare" ''
    set -eu

    case "$USER" in
    ${accountCases}
      *)
        # La sesión actual no pertenece a este host declarativo.
        exit 0
        ;;
    esac

    config_home="$HOME/.config"
    state_home="$HOME/.local/state"

    if [ -n "''${XDG_CONFIG_HOME:-}" ]; then
      config_home="$XDG_CONFIG_HOME"
    fi

    if [ -n "''${XDG_STATE_HOME:-}" ]; then
      state_home="$XDG_STATE_HOME"
    fi

    korunix_state="$state_home/korunix"
    mkdir -p "$korunix_state"

    ${lib.optionalString (lib.elem "spotify" config.korunix.applications) ''
      # Una implementación anterior creó un lanzador personal que apuntaba a
      # ~/.local/share/korunix/spotify. Los archivos personales tienen prioridad
      # sobre el .desktop declarativo y evitaban el selector por sesión. Solo
      # retiramos el archivo si todavía coincide exactamente con aquel residuo;
      # cualquier lanzador personalizado por la persona se conserva.
      data_home="$HOME/.local/share"

      if [ -n "''${XDG_DATA_HOME:-}" ]; then
        data_home="$XDG_DATA_HOME"
      fi

      legacy_spotify="$HOME/.local/share/korunix/spotify/spotify"
      legacy_launcher="$data_home/applications/spotify.desktop"

      if [ -f "$legacy_launcher" ] \
          && grep -Fqx "TryExec=$legacy_spotify" "$legacy_launcher" \
          && grep -Fqx "Exec=$legacy_spotify %U" "$legacy_launcher"
      then
        launcher_backup="$korunix_state/backups/spotify-launcher"
        mkdir -p "$launcher_backup"

        mv \
          "$legacy_launcher" \
          "$launcher_backup/spotify.desktop.$(date +%Y%m%d-%H%M%S)-$$"
      fi
    ''}

    # Un enlace se puede actualizar porque sabemos que pertenece a Korunix. Un
    # archivo normal se conserva: puede haber sido editado manualmente.
    ensure_link() {
      target="$1"
      source="$2"
      mkdir -p "$(dirname "$target")"

      if [ -L "$target" ]; then
        ln -sfn "$source" "$target"
      elif [ ! -e "$target" ]; then
        ln -s "$source" "$target"
      fi
    }

    # Fetch tiene una configuración común y legible dentro de .korunix.
    ensure_link "$config_home/fetch/config" "/etc/korunix/fetch.conf"

    # La configuración antigua de Fastfetch era de Home Manager. Solo retiramos
    # enlaces simbólicos que claramente pertenecían a ese mecanismo.
    if [ -L "$config_home/fastfetch/config.jsonc" ]; then
      rm -f "$config_home/fastfetch/config.jsonc"
    fi

    # La foto de perfil es una propiedad del usuario y se comparte con GDM,
    # Noctalia y cualquier componente que respete ~/.face.
    if [ -n "$KORUNIX_AVATAR_SOURCE" ]; then
      if [ -L "$HOME/.face" ]; then
        ln -sfn "$KORUNIX_AVATAR_SOURCE" "$HOME/.face"
      elif [ ! -e "$HOME/.face" ]; then
        ln -s "$KORUNIX_AVATAR_SOURCE" "$HOME/.face"
      fi
    fi

    # Las integraciones de aplicaciones conservan un archivo manual si existe.
    if [ -e /etc/korunix/noctalia/themes/obsidian/obsidian.css ]; then
      ensure_link \
        "$HOME/.obsidian/snippets/noctalia.css" \
        "/etc/korunix/noctalia/themes/obsidian/obsidian.css"
    fi

    if [ -e /etc/korunix/noctalia/themes/heroic/heroic.css ]; then
      ensure_link \
        "$config_home/heroic/themes/noctalia.css" \
        "/etc/korunix/noctalia/themes/heroic/heroic.css"
    fi

    # El antiguo config.fish de Home Manager puede seguir como enlace. Fish ya
    # recibe la configuración desde NixOS, por lo que ese enlace deja de servir.
    if [ -L "$config_home/fish/config.fish" ]; then
      rm -f "$config_home/fish/config.fish"
    fi

    # Fcitx5 es backend del host, pero el grupo pertenece a cada persona.
    # Si cualquier persona del host necesita Fcitx5, todas las cuentas Korunix
    # reciben al menos su teclado normal para evitar caer en keyboard-us.
    #
    # Solo sustituimos un archivo que siga siendo exactamente el último generado
    # por Korunix. Una edición manual se conserva.
    fcitx_dir="$config_home/fcitx5"
    fcitx_target="$fcitx_dir/profile"
    fcitx_hash="$korunix_state/fcitx5-profile.sha256"
    fcitx_tmp="$korunix_state/fcitx5-profile.new"

    if [ "$KORUNIX_FCITX_ENABLED" = "1" ]; then
      mkdir -p "$fcitx_dir"

      default_im="$KORUNIX_FCITX_KEYBOARD"

      if [ -n "$KORUNIX_FCITX_METHODS" ]; then
        default_im="''${KORUNIX_FCITX_METHODS%%,*}"
      fi

      {
        echo "[Groups/0]"
        echo "Name=Korunix"
        echo "Default Layout=${fcitxDefaultLayout}"
        echo "DefaultIM=$default_im"

        echo
        echo "[Groups/0/Items/0]"
        echo "Name=$KORUNIX_FCITX_KEYBOARD"
        echo "Layout="

        old_ifs="$IFS"
        IFS=","
        index=1

        for method in $KORUNIX_FCITX_METHODS; do
          [ -n "$method" ] || continue

          echo
          echo "[Groups/0/Items/$index]"
          echo "Name=$method"
          echo "Layout="

          index=$((index + 1))
        done

        IFS="$old_ifs"

        echo
        echo "[GroupOrder]"
        echo "0=Korunix"
      } > "$fcitx_tmp"

      new_hash="$(sha256sum "$fcitx_tmp" | cut -d' ' -f1)"

      if [ ! -e "$fcitx_target" ] && [ ! -L "$fcitx_target" ]; then
        mv "$fcitx_tmp" "$fcitx_target"
        printf '%s\n' "$new_hash" > "$fcitx_hash"
      elif [ ! -L "$fcitx_target" ] \
          && [ -f "$fcitx_hash" ] \
          && [ -f "$fcitx_target" ]
      then
        old_hash="$(cat "$fcitx_hash")"
        current_hash="$(sha256sum "$fcitx_target" | cut -d' ' -f1)"

        if [ "$current_hash" = "$old_hash" ]; then
          mv "$fcitx_tmp" "$fcitx_target"
          printf '%s\n' "$new_hash" > "$fcitx_hash"
        fi
      fi

      if [ -e "$fcitx_tmp" ]; then
        rm -f "$fcitx_tmp"
        printf '%s\n' \
          "Korunix preservó ~/.config/fcitx5/profile porque contiene cambios manuales." \
          >&2
      fi
    else
      # Al dejar de necesitar Fcitx5 retiramos únicamente una copia que siga
      # siendo idéntica a la última generada por Korunix.
      if [ ! -L "$fcitx_target" ] \
          && [ -f "$fcitx_hash" ] \
          && [ -f "$fcitx_target" ]
      then
        old_hash="$(cat "$fcitx_hash")"
        current_hash="$(sha256sum "$fcitx_target" | cut -d' ' -f1)"

        if [ "$current_hash" = "$old_hash" ]; then
          rm -f "$fcitx_target" "$fcitx_hash"
        fi
      fi

      rm -f "$fcitx_tmp"
    fi

    if [ ! -e /etc/korunix/noctalia/config.toml ]; then
      exit 0
    fi

    pictures_dir="$(${pkgs.xdg-user-dirs}/bin/xdg-user-dir PICTURES 2>/dev/null || true)"
    if [ -z "$pictures_dir" ]; then
      pictures_dir="$HOME/Pictures"
    fi

    case "$KORUNIX_LANGUAGE" in
      es*) screenshots_name="Capturas de pantalla" ;;
      *) screenshots_name="Screenshots" ;;
    esac

    screenshots_dir="$pictures_dir/$screenshots_name"
    mkdir -p "$screenshots_dir"

    noctalia_dir="$config_home/noctalia"
    noctalia_target="$noctalia_dir/config.toml"
    noctalia_hash="$korunix_state/noctalia-config.sha256"
    noctalia_tmp="$korunix_state/noctalia-config.toml.new"
    mkdir -p "$noctalia_dir"

    # La plantilla contiene marcadores únicamente para valores que dependen de la
    # persona o de XDG. El resto sigue siendo una única fuente común en .korunix.
    avatar_value=""
    if [ -n "$KORUNIX_AVATAR_SOURCE" ]; then
      avatar_value="~/.face"
    fi

    escape_sed() {
      printf '%s' "$1" | sed 's/[&|\\]/\\&/g'
    }

    sed \
      -e "s|@KORUNIX_AVATAR@|$(escape_sed "$avatar_value")|g" \
      -e "s|@KORUNIX_SCREENSHOTS@|$(escape_sed "$screenshots_dir")|g" \
      -e "s|@KORUNIX_NOCTALIA@|/etc/korunix/noctalia|g" \
      /etc/korunix/noctalia/config.toml \
      > "$noctalia_tmp"

    # Spotify solo recibe la plantilla cuando forma parte de las aplicaciones
    # elegidas. El marcador se retira siempre para que el TOML final sea normal.
    if [ -e /etc/korunix/noctalia/spicetify-template.toml ]; then
      sed -i \
        '/# @KORUNIX_SPOTIFY_TEMPLATE@/r /etc/korunix/noctalia/spicetify-template.toml' \
        "$noctalia_tmp"
    fi

    sed -i \
      '/# @KORUNIX_SPOTIFY_TEMPLATE@/d' \
      "$noctalia_tmp"

    new_hash="$(sha256sum "$noctalia_tmp" | cut -d' ' -f1)"

    if [ -L "$noctalia_target" ]; then
      # Los enlaces heredados de Home Manager pertenecen a Korunix y pueden ser
      # sustituidos por el nuevo archivo generado para esta persona.
      rm -f "$noctalia_target"
    fi

    if [ ! -e "$noctalia_target" ]; then
      mv "$noctalia_tmp" "$noctalia_target"
      printf '%s\n' "$new_hash" > "$noctalia_hash"
      exit 0
    fi

    if [ -f "$noctalia_hash" ]; then
      old_hash="$(cat "$noctalia_hash")"
      current_hash="$(sha256sum "$noctalia_target" | cut -d' ' -f1)"

      if [ "$current_hash" = "$old_hash" ]; then
        mv "$noctalia_tmp" "$noctalia_target"
        printf '%s\n' "$new_hash" > "$noctalia_hash"
        exit 0
      fi
    fi

    # Un archivo normal cuyo hash ya no coincide puede haber sido editado a mano.
    # Korunix lo preserva en vez de sobrescribirlo silenciosamente.
    rm -f "$noctalia_tmp"
    printf '%s\n' \
      "Korunix preservó ~/.config/noctalia/config.toml porque contiene cambios manuales." \
      >&2
  '';
in {
  config = lib.mkIf cfg.enable {
    # La clave privada sigue fuera de Nix/Git. Korunix solo declara qué archivo
    # local debe usar cada cuenta y OpenSSH la conserva en su agente tras usarla.
    programs.ssh.extraConfig = githubSshConfig;

    assertions = [
      {
        assertion = cfg.users != [];
        message = "Korunix necesita al menos una persona asignada al host.";
      }
      {
        assertion = missingSettingsIds == [];
        message =
          "Cada persona del host necesita estado local en userSettings. Faltan: "
          + lib.concatStringsSep ", " missingSettingsIds;
      }
      {
        assertion = extraSettingsIds == [];
        message =
          "Hay estado local de usuarios que no pertenecen al host: "
          + lib.concatStringsSep ", " extraSettingsIds;
      }
      {
        assertion = administratorIds != [];
        message = "Korunix no permite dejar un host sin ningún administrador declarado.";
      }
      {
        assertion =
          builtins.length accountNames
          == builtins.length (lib.unique accountNames);
        message = "Dos perfiles Korunix no pueden administrar la misma cuenta UNIX en un host.";
      }
      {
        assertion = unknownCapabilities == [];
        message =
          "Korunix no conoce estas capacidades portables: "
          + lib.concatStringsSep ", " unknownCapabilities;
      }
      {
        assertion = unknownDeferredCapabilities == [];
        message =
          "Korunix no conoce estas capacidades aplazadas: "
          + lib.concatStringsSep ", " unknownDeferredCapabilities;
      }
      {
        assertion = invalidDeferredCapabilities == [];
        message =
          "Un host solo puede aplazar capacidades pedidas por el perfil: "
          + lib.concatStringsSep ", " invalidDeferredCapabilities;
      }
      {
        assertion = unknownInputMethods == [];
        message =
          "Korunix no conoce estos métodos de entrada portables: "
          + lib.concatStringsSep ", " unknownInputMethods;
      }
      {
        assertion = unknownDeferredInputMethods == [];
        message =
          "Korunix no conoce estos métodos de entrada aplazados: "
          + lib.concatStringsSep ", " unknownDeferredInputMethods;
      }
      {
        assertion = invalidDeferredInputMethods == [];
        message =
          "Un host solo puede aplazar métodos pedidos por el perfil: "
          + lib.concatStringsSep ", " invalidDeferredInputMethods;
      }
      {
        assertion =
          !androidRequested
          || lib.elem "android-tools" config.korunix.applications;
        message = "La capacidad Android activa necesita android-tools en este host.";
      }
    ];

    # GNOME propone IBus mediante mkDefault. Mientras nadie necesite un método
    # avanzado dejamos ese comportamiento exactamente intacto. Cuando exista una
    # selección efectiva, Korunix elige Fcitx5 explícitamente para todo el host.
    i18n.inputMethod = lib.mkIf advancedInputMethodsEnabled {
      enable = true;
      type = "fcitx5";

      fcitx5 = {
        addons = fcitxAddons;

        # GTK/Qt usan sus módulos Fcitx5. El compositor sigue siendo dueño del
        # teclado Wayland/XKB del host.
        waylandFrontend = false;
      };
    };

    # Contrato interno legible para los backends de terminal y la GUI futura.
    # No contiene nombres de paquetes dentro del perfil portable de la persona.
    environment.etc."korunix/input-methods.json".text = builtins.toJSON {
      schemaVersion = 1;

      backend =
        if advancedInputMethodsEnabled
        then "fcitx5"
        else "desktop-default";

      launcher =
        if advancedInputMethodsEnabled
        then "xdg-autostart"
        else null;

      keyboard = {
        engine = fcitxKeyboard;
        layout = fcitxDefaultLayout;
      };

      catalog = inputMethodCatalog;

      people =
        map (
          userId: {
            id = userId;
            requested = requestedInputMethodsFor userId;
            deferred = deferredInputMethodsFor userId;
            effective = usableInputMethodsFor userId;
          }
        )
        cfg.users;
    };

    # mutableUsers conserva las contraseñas creadas por Calamares. Korunix declara
    # identidad y capacidades, pero no coloca hashes de contraseñas en Git.
    users.mutableUsers = true;
    users.users = usersConfig;

    systemd.user.services.korunix-user-prepare = {
      description = "Prepara los archivos personales administrados por Korunix";

      # UWSM enlaza graphical-session-pre, graphical-session y el autostart XDG.
      # Preparar aquí evita una carrera entre el profile personal y el lanzador
      # oficial org.fcitx.Fcitx5.desktop.
      wantedBy = [
        "graphical-session-pre.target"
      ];

      before = [
        "graphical-session.target"
        "xdg-desktop-autostart.target"
        "noctalia.service"
      ];

      path = [
        pkgs.coreutils
        pkgs.gnugrep
        pkgs.gnused
        pkgs.xdg-user-dirs
      ];

      serviceConfig = {
        Type = "oneshot";
        ExecStart = prepareUser;
      };
    };
  };
}
