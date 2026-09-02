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
    path = korunixContext.personasPath + "/${userId}.nix";
  in
    if builtins.pathExists path
    then import path
    else throw "El host ${cfg.hostId} utiliza ${userId}, pero falta configuracion/personas/${userId}.nix.";

  profiles = lib.genAttrs cfg.users loadUser;

  roleModel = import ./roles.nix {inherit lib pkgs;};

  enabledDesktops =
    lib.unique ([cfg.desktop.primary] ++ cfg.desktop.additional);

  plasmaEnabled = lib.elem "plasma" enabledDesktops;

  settingsFor = userId: cfg.userSettings.${userId} or {};

  defaultRolesFor = userId:
    profiles.${userId}.defaultRoles or {};

  browserRoleFor = userId:
    (defaultRolesFor userId).browser or null;

  plasmaTextEditorRoleFor = userId:
    (defaultRolesFor userId).plasmaTextEditor or null;

  browserEffectiveFor = userId: let
    requested = browserRoleFor userId;
  in
    if requested != null && lib.elem requested cfg.applications
    then requested
    else null;

  defaultRoleKeysFor = userId:
    builtins.attrNames (defaultRolesFor userId);

  knownDefaultRoleKeys = [
    "browser"
    "plasmaTextEditor"
  ];

  unknownDefaultRoleKeys =
    lib.concatMap
    (userId:
      map
      (key: "${userId}:${key}")
      (lib.filter
        (key: !(lib.elem key knownDefaultRoleKeys))
        (defaultRoleKeysFor userId)))
    cfg.users;

  invalidBrowserRoles =
    lib.concatMap
    (userId: let
      value = browserRoleFor userId;
    in
      lib.optional
      (value != null && !(lib.elem value roleModel.browserChoices))
      "${userId}:${toString value}")
    cfg.users;

  invalidPlasmaTextEditorRoles =
    lib.concatMap
    (userId: let
      value = plasmaTextEditorRoleFor userId;
    in
      lib.optional
      (value != null && !(lib.elem value roleModel.plasmaTextEditorChoices))
      "${userId}:${toString value}")
    cfg.users;

  plasmaEditorRequested =
    plasmaEnabled
    && lib.any
    (userId: plasmaTextEditorRoleFor userId != null)
    cfg.users;

  mimeDefaultsForUser = userId:
    lib.listToAttrs
    (map
      (desktop: {
        name = roleModel.desktopMimeFileNames.${desktop};
        value = roleModel.mimeDefaultsFor {
          inherit desktop;
          browser = browserEffectiveFor userId;
          plasmaTextEditor = plasmaTextEditorRoleFor userId;
        };
      })
      enabledDesktops);

  roleStateForUser = userId: let
    browserRequested = browserRoleFor userId;
    browserEffective = browserEffectiveFor userId;
    plasmaEditorRequestedByUser = plasmaTextEditorRoleFor userId;
  in {
    id = userId;

    requested = {
      browser = browserRequested;
      plasmaTextEditor = plasmaEditorRequestedByUser;
    };

    effective = {
      browser = browserEffective;
      plasmaTextEditor =
        if plasmaEnabled
        then plasmaEditorRequestedByUser
        else null;
    };

    deferred = {
      browser =
        browserRequested
        != null
        && browserEffective == null;
      plasmaTextEditor = false;
    };

    needsChoice = {
      browser = browserRequested == null;
      plasmaTextEditor =
        plasmaEnabled
        && plasmaEditorRequestedByUser == null;
    };

    mimeFiles = mimeDefaultsForUser userId;
  };

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
        mimeDefaults = builtins.toJSON (mimeDefaultsForUser userId);
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
          KORUNIX_MIME_DEFAULTS=${lib.escapeShellArg mimeDefaults}
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


    # Los archivos específicos del escritorio tienen prioridad sobre el
    # mimeapps.list general. Korunix modifica únicamente Default Applications
    # de los roles que administra y conserva cualquier asociación ajena.
    ${pkgs.python3}/bin/python3 - \
      "$config_home" \
      "$korunix_state" \
      "$KORUNIX_MIME_DEFAULTS" \
      <<'PY'
    from __future__ import annotations

    import json
    import os
    import re
    import shutil
    import sys
    import tempfile
    import time
    from pathlib import Path

    config_home = Path(sys.argv[1])
    state_home = Path(sys.argv[2])
    files = json.loads(sys.argv[3])

    header = re.compile(r"^[ \t]*\[Default Applications\][ \t]*$", re.M)


    def merge_defaults(source: str, defaults: dict[str, str]) -> str:
        if not defaults:
            return source

        match = header.search(source)

        if match is None:
            prefix = source.rstrip()
            if prefix:
                prefix += "\n\n"
            section = "[Default Applications]\n"
            section += "".join(
                f"{mime}={desktop}\n"
                for mime, desktop in sorted(defaults.items())
            )
            return prefix + section

        following = re.search(r"(?m)^[ \t]*\[", source[match.end():])
        end = match.end() + following.start() if following else len(source)

        before = source[:match.end()]
        body = source[match.end():end]
        after = source[end:]

        lines = body.splitlines(keepends=True)
        managed = dict(defaults)
        seen: set[str] = set()
        result: list[str] = []

        assignment = re.compile(r"^([ \t]*)([^#;\s][^=]*?)[ \t]*=(.*)$")

        for line in lines:
            raw = line.rstrip("\r\n")
            match_line = assignment.match(raw)

            if match_line is None:
                result.append(line)
                continue

            key = match_line.group(2).strip()

            if key not in managed:
                result.append(line)
                continue

            if key in seen:
                continue

            newline = "\r\n" if line.endswith("\r\n") else "\n"
            result.append(
                f"{match_line.group(1)}{key}={managed[key]}{newline}"
            )
            seen.add(key)

        missing = [
            key
            for key in sorted(managed)
            if key not in seen
        ]

        if missing:
            if result and not result[-1].endswith(("\n", "\r\n")):
                result[-1] += "\n"

            if not result or result[-1].strip():
                result.append("\n")

            result.extend(
                f"{key}={managed[key]}\n"
                for key in missing
            )

        return before + "".join(result) + after


    for filename, defaults in files.items():
        if not re.fullmatch(
            r"(?:niri|hyprland|x-cinnamon|kde)-mimeapps\.list",
            filename,
        ):
            raise SystemExit(
                f"Korunix: nombre de archivo MIME no permitido: {filename}"
            )

        target = config_home / filename
        target.parent.mkdir(parents=True, exist_ok=True)

        original = (
            target.read_text(encoding="utf-8")
            if target.exists()
            else ""
        )

        merged = merge_defaults(original, defaults)
        if merged and not merged.endswith("\n"):
            merged += "\n"

        if merged == original:
            continue

        if target.exists():
            backup_dir = state_home / "backups" / "mime"
            backup_dir.mkdir(parents=True, exist_ok=True)
            stamp = f"{time.time_ns()}-{os.getpid()}"
            shutil.copy2(
                target,
                backup_dir / f"{filename}.{stamp}",
            )

        descriptor, temporary_name = tempfile.mkstemp(
            prefix=f".{filename}.korunix-",
            dir=target.parent,
            text=True,
        )
        temporary = Path(temporary_name)

        try:
            with os.fdopen(descriptor, "w", encoding="utf-8") as output:
                output.write(merged)
                output.flush()
                os.fsync(output.fileno())

            if target.exists():
                os.chmod(temporary, target.stat().st_mode)

            os.replace(temporary, target)
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    PY

    ${lib.optionalString (lib.elem "figma-linux-next" config.korunix.applications) ''
      # El módulo de Figma Linux Next declara correctamente figma:// a nivel del
      # sistema, pero una asociación personal heredada tiene prioridad. Korunix
      # migra únicamente el valor exacto que dejó el cliente antiguo; cualquier
      # elección distinta de la persona se conserva.
      ${pkgs.python3}/bin/python3 - \
        "$config_home/mimeapps.list" \
        "$korunix_state" \
        <<'PY'
      from __future__ import annotations

      import os
      import re
      import shutil
      import sys
      import tempfile
      import time
      from pathlib import Path

      target = Path(sys.argv[1])
      state_home = Path(sys.argv[2])
      mime = "x-scheme-handler/figma"
      legacy_figma = "figma-linux.desktop"
      current_figma = "figma-linux-next.desktop"

      if not target.is_file():
          raise SystemExit(0)

      source = target.read_text(encoding="utf-8")
      header = re.compile(
          r"^[ \t]*\[Default Applications\][ \t]*$",
          re.M,
      )
      section = header.search(source)

      if section is None:
          raise SystemExit(0)

      following = re.search(
          r"(?m)^[ \t]*\[",
          source[section.end():],
      )
      end = (
          section.end() + following.start()
          if following
          else len(source)
      )

      before = source[:section.end()]
      body = source[section.end():end]
      after = source[end:]
      lines = body.splitlines(keepends=True)

      assignment = re.compile(
          r"^([ \t]*)([^#;\s][^=]*?)[ \t]*=(.*)$"
      )

      matches: list[tuple[int, re.Match[str], str, str]] = []

      for index, line in enumerate(lines):
          raw = line.rstrip("\r\n")
          match = assignment.match(raw)

          if match is None:
              continue

          key = match.group(2).strip()
          if key != mime:
              continue

          raw_value = match.group(3).strip()
          semicolon = ";" if raw_value.endswith(";") else ""
          normalized = raw_value[:-1].strip() if semicolon else raw_value

          matches.append((index, match, normalized, semicolon))

      # Un archivo ambiguo o personalizado no se reinterpreta automáticamente.
      if len(matches) != 1:
          raise SystemExit(0)

      index, match, normalized, semicolon = matches[0]

      if normalized != legacy_figma:
          raise SystemExit(0)

      newline = "\r\n" if lines[index].endswith("\r\n") else "\n"
      lines[index] = (
          f"{match.group(1)}{mime}={current_figma}{semicolon}{newline}"
      )

      migrated = before + "".join(lines) + after

      if migrated == source:
          raise SystemExit(0)

      backup_dir = state_home / "backups" / "mime"
      backup_dir.mkdir(parents=True, exist_ok=True)
      stamp = f"{time.time_ns()}-{os.getpid()}"
      shutil.copy2(
          target,
          backup_dir / f"mimeapps.list.figma-legacy.{stamp}",
      )

      descriptor, temporary_name = tempfile.mkstemp(
          prefix=".mimeapps.list.korunix-figma-",
          dir=target.parent,
          text=True,
      )
      temporary = Path(temporary_name)

      try:
          with os.fdopen(descriptor, "w", encoding="utf-8") as output:
              output.write(migrated)
              output.flush()
              os.fsync(output.fileno())

          os.chmod(temporary, target.stat().st_mode)
          os.replace(temporary, target)

          directory = os.open(target.parent, os.O_RDONLY)
          try:
              os.fsync(directory)
          finally:
              os.close(directory)
      except BaseException:
          temporary.unlink(missing_ok=True)
          raise
      PY
    ''}

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

    # Este fragmento pertenece a Korunix y se carga junto a config.toml sin
    # sobrescribir las preferencias personales guardadas por Noctalia.
    ensure_link       "$config_home/noctalia/30-korunix-gtk4-live.toml"       "/etc/korunix/noctalia/gtk4-live.toml"

    # Fetch ya usa un XDG privado administrado por el wrapper del sistema. Se
    # retira únicamente el enlace heredado exacto que creó Korunix; un archivo
    # normal o un enlace hacia otro destino se conserva como dato de la persona.
    fetch_legacy="$config_home/fetch/config"
    if [ -L "$fetch_legacy" ] \
        && [ "$(readlink "$fetch_legacy")" = "/etc/korunix/fetch.conf" ]
    then
      rm -f "$fetch_legacy"
    fi

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

    pictures_dir="$(${pkgs.xdg-user-dirs}/bin/xdg-user-dir PICTURES 2>/dev/null || true)"
    if [ -z "$pictures_dir" ]; then
      pictures_dir="$HOME/Pictures"
    fi

    case "$KORUNIX_LANGUAGE" in
      be-Latn*)
        screenshots_name="Zdymki ekrana"
        screenshot_pattern="Zdymak ekrana ad %Y-%m-%d %H-%M-%S"
        ;;
      be*)
        screenshots_name="Здымкі экрана"
        screenshot_pattern="Здымак экрана ад %Y-%m-%d %H-%M-%S"
        ;;
      ca*)
        screenshots_name="Captures de pantalla"
        screenshot_pattern="Captura de pantalla del %Y-%m-%d %H-%M-%S"
        ;;
      cs*)
        screenshots_name="Snímky obrazovky"
        screenshot_pattern="Snímek obrazovky z %Y-%m-%d %H-%M-%S"
        ;;
      de*)
        screenshots_name="Bildschirmfotos"
        screenshot_pattern="Bildschirmfoto vom %Y-%m-%d um %H-%M-%S"
        ;;
      es*)
        screenshots_name="Capturas de pantalla"
        screenshot_pattern="Captura de pantalla del %Y-%m-%d %H-%M-%S"
        ;;
      fr*)
        screenshots_name="Captures d’écran"
        screenshot_pattern="Capture d’écran du %Y-%m-%d à %H-%M-%S"
        ;;
      gl-ES*)
        screenshots_name="Capturas de pantalla"
        screenshot_pattern="Captura de pantalla do %Y-%m-%d %H-%M-%S"
        ;;
      hu*)
        screenshots_name="Képernyőképek"
        screenshot_pattern="Képernyőkép %Y-%m-%d %H-%M-%S"
        ;;
      it*)
        screenshots_name="Schermate"
        screenshot_pattern="Schermata del %Y-%m-%d alle %H-%M-%S"
        ;;
      ko*)
        screenshots_name="스크린샷"
        screenshot_pattern="%Y-%m-%d %H-%M-%S 스크린샷"
        ;;
      ku*)
        screenshots_name="Wêneyên dîmenderê"
        screenshot_pattern="Wêneyê dîmenderê %Y-%m-%d %H-%M-%S"
        ;;
      nl*)
        screenshots_name="Schermafbeeldingen"
        screenshot_pattern="Schermafbeelding van %Y-%m-%d om %H-%M-%S"
        ;;
      nn*)
        screenshots_name="Skjermbilete"
        screenshot_pattern="Skjermbilete frå %Y-%m-%d %H-%M-%S"
        ;;
      pl*)
        screenshots_name="Zrzuty ekranu"
        screenshot_pattern="Zrzut ekranu z %Y-%m-%d %H-%M-%S"
        ;;
      pt-BR*)
        screenshots_name="Capturas de tela"
        screenshot_pattern="Captura de tela de %Y-%m-%d %H-%M-%S"
        ;;
      ru*)
        screenshots_name="Снимки экрана"
        screenshot_pattern="Снимок экрана от %Y-%m-%d %H-%M-%S"
        ;;
      sv*)
        screenshots_name="Skärmbilder"
        screenshot_pattern="Skärmbild från %Y-%m-%d %H-%M-%S"
        ;;
      tr*)
        screenshots_name="Ekran görüntüleri"
        screenshot_pattern="%Y-%m-%d %H-%M-%S ekran görüntüsü"
        ;;
      uk-UA*)
        screenshots_name="Знімки екрана"
        screenshot_pattern="Знімок екрана від %Y-%m-%d %H-%M-%S"
        ;;
      vi*)
        screenshots_name="Ảnh chụp màn hình"
        screenshot_pattern="Ảnh chụp màn hình lúc %Y-%m-%d %H-%M-%S"
        ;;
      zh-Hans*)
        screenshots_name="屏幕截图"
        screenshot_pattern="%Y-%m-%d %H-%M-%S 的屏幕截图"
        ;;
      *)
        screenshots_name="Screenshots"
        screenshot_pattern="Screenshot from %Y-%m-%d %H-%M-%S"
        ;;
    esac

    screenshots_dir="$pictures_dir/$screenshots_name"
    mkdir -p "$screenshots_dir"

    # Noctalia tiene una capa declarativa y otra guardada por su interfaz.
    # Esta función modifica únicamente la política que Korunix administra y
    # conserva el resto del TOML, incluso cuando sus secciones están sangradas.
    merge_noctalia_policy() {
      local target="$1"

      ${pkgs.python3}/bin/python3 - \
        "$target" \
        "$screenshots_dir" \
        "$screenshot_pattern" \
        <<'PY'
    from __future__ import annotations

    import json
    import os
    import re
    import sys
    import tempfile
    import tomllib
    from pathlib import Path


    link = Path(sys.argv[1])
    target = link.resolve(strict=True) if link.is_symlink() else link
    screenshots_dir = sys.argv[2]
    screenshot_pattern = sys.argv[3]
    original = target.read_text(encoding="utf-8")
    source = original


    def toml_string(value: str) -> str:
        return json.dumps(value, ensure_ascii=False)


    def section_bounds(text: str, section: str) -> tuple[int, int] | None:
        header = re.compile(
            rf"(?m)^[ \t]*\[{re.escape(section)}\][ \t]*(?:#.*)?$"
        )
        match = header.search(text)
        if match is None:
            return None

        following = re.search(r"(?m)^[ \t]*\[", text[match.end():])
        end = match.end() + following.start() if following else len(text)
        return match.start(), end


    def replace_key_in_section(
        text: str,
        section: str,
        key: str,
        value: str,
    ) -> str:
        bounds = section_bounds(text, section)
        line = f"{key} = {toml_string(value)}"

        if bounds is None:
            return text.rstrip() + f"\n\n[{section}]\n{line}\n"

        start, end = bounds
        body = text[start:end]
        key_pattern = re.compile(
            rf"(?m)^([ \t]*){re.escape(key)}[ \t]*=.*$"
        )

        if key_pattern.search(body):
            body = key_pattern.sub(
                lambda match: f"{match.group(1)}{line}",
                body,
                count=1,
            )
        else:
            value_line = re.search(r"(?m)^([ \t]*)[A-Za-z0-9_-]+[ \t]*=", body)
            indent = value_line.group(1) if value_line else ""
            body = body.rstrip() + f"\n{indent}{line}\n\n"

        return text[:start] + body + text[end:]


    def remove_section(text: str, section: str) -> str:
        bounds = section_bounds(text, section)
        if bounds is None:
            return text

        start, end = bounds
        return text[:start].rstrip() + "\n\n" + text[end:].lstrip()


    source = replace_key_in_section(
        source,
        "shell.screenshot",
        "directory",
        screenshots_dir,
    )
    source = replace_key_in_section(
        source,
        "shell.screenshot",
        "filename_pattern",
        screenshot_pattern,
    )

    # Versiones anteriores de Korunix añadían dos filtros para ocultar un
    # notificador auxiliar de Cinnamon. Ya no forman parte de la política de
    # Noctalia; solo se eliminan de configuraciones heredadas.
    legacy_filters = (
        "korunix-system-failure-en",
        "korunix-system-failure-es",
    )

    for name in legacy_filters:
        source = remove_section(source, f"notification.filter.{name}")

    source = re.sub(
        r"(?m)^[ \t]*# Filtros administrados por Korunix: "
        r"ocultan solo el auxiliar gráfico de Cinnamon\.[ \t]*\n?",
        "",
        source,
    )
    source = source.rstrip() + "\n"

    parsed = tomllib.loads(source)
    screenshot = parsed["shell"]["screenshot"]
    if screenshot["directory"] != screenshots_dir:
        raise SystemExit("Korunix: no se pudo fusionar el directorio de capturas.")
    if screenshot["filename_pattern"] != screenshot_pattern:
        raise SystemExit("Korunix: no se pudo fusionar el nombre de las capturas.")

    if source == original:
        raise SystemExit(0)

    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{target.name}.korunix-",
        dir=target.parent,
        text=True,
    )
    temporary = Path(temporary_name)

    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            output.write(source)
            output.flush()
            os.fsync(output.fileno())

        os.chmod(temporary, target.stat().st_mode)
        temporary.replace(target)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise
    PY
    }

    # Noctalia y el capturador propio de Niri comparten el mismo destino XDG.
    # Niri admite archivos incluidos desde el home, así que esta ruta puede ser
    # específica de cada persona sin introducir un /home fijo en la configuración
    # común del sistema.
    niri_dir="$config_home/niri"
    niri_screenshots_target="$niri_dir/korunix-screenshots.kdl"
    niri_screenshots_hash="$korunix_state/niri-screenshots.sha256"
    niri_screenshots_tmp="$korunix_state/niri-screenshots.kdl.new"
    mkdir -p "$niri_dir"

    escape_kdl() {
      printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
    }

    printf 'screenshot-path "%s/%s.png"\n' \
      "$(escape_kdl "$screenshots_dir")" \
      "$(escape_kdl "$screenshot_pattern")" \
      > "$niri_screenshots_tmp"

    niri_screenshots_new_hash="$(sha256sum "$niri_screenshots_tmp" | cut -d' ' -f1)"

    if [ -L "$niri_screenshots_target" ]; then
      rm -f "$niri_screenshots_target"
    fi

    if [ ! -e "$niri_screenshots_target" ]; then
      mv "$niri_screenshots_tmp" "$niri_screenshots_target"
      printf '%s\n' "$niri_screenshots_new_hash" > "$niri_screenshots_hash"
    elif [ -f "$niri_screenshots_hash" ] \
        && [ -f "$niri_screenshots_target" ]
    then
      niri_screenshots_old_hash="$(cat "$niri_screenshots_hash")"
      niri_screenshots_current_hash="$(sha256sum "$niri_screenshots_target" | cut -d' ' -f1)"

      if [ "$niri_screenshots_current_hash" = "$niri_screenshots_old_hash" ]; then
        mv "$niri_screenshots_tmp" "$niri_screenshots_target"
        printf '%s\n' "$niri_screenshots_new_hash" > "$niri_screenshots_hash"
      fi
    fi

    if [ -e "$niri_screenshots_tmp" ]; then
      rm -f "$niri_screenshots_tmp"
      printf '%s\n' \
        "Korunix preservó ~/.config/niri/korunix-screenshots.kdl porque contiene cambios manuales." \
        >&2
    fi

    # settings.toml se carga después de config.toml. La misma política debe
    # quedar allí para que una preferencia antigua no anule la ruta XDG.
    noctalia_state_home="$(printenv XDG_STATE_HOME 2>/dev/null || true)"
    if [ -z "$noctalia_state_home" ]; then
      noctalia_state_home="$HOME/.local/state"
    fi

    noctalia_settings_target="$noctalia_state_home/noctalia/settings.toml"
    if [ -f "$noctalia_settings_target" ]; then
      merge_noctalia_policy "$noctalia_settings_target"
    fi

    if [ ! -e /etc/korunix/noctalia/config.toml ]; then
      exit 0
    fi

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
      -e "s|@KORUNIX_SCREENSHOT_PATTERN@|$(escape_sed "$screenshot_pattern")|g" \
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

    # Un archivo cuyo hash ya no coincide contiene preferencias personales.
    # Korunix conserva todo ese contenido y fusiona únicamente la política de
    # capturas. También elimina los dos filtros heredados que versiones anteriores
    # añadían para un notificador auxiliar de Cinnamon.
    merge_noctalia_policy "$noctalia_target"

    # El hash anterior se conserva deliberadamente: el archivo continúa siendo
    # propiedad de la persona y las futuras preparaciones repetirán la fusión
    # acotada en vez de sustituir sus preferencias.
    rm -f "$noctalia_tmp"
    printf '%s\n' \
      "Korunix conservó las preferencias de Noctalia y actualizó solo su política administrada." \
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
        assertion = unknownDefaultRoleKeys == [];
        message =
          "Korunix no conoce estos roles predeterminados portables: "
          + lib.concatStringsSep ", " unknownDefaultRoleKeys;
      }
      {
        assertion = invalidBrowserRoles == [];
        message =
          "Korunix solo permite Firefox o Google Chrome como navegador predeterminado: "
          + lib.concatStringsSep ", " invalidBrowserRoles;
      }
      {
        assertion = invalidPlasmaTextEditorRoles == [];
        message =
          "Plasma solo permite KWrite o Kate como editor predeterminado: "
          + lib.concatStringsSep ", " invalidPlasmaTextEditorRoles;
      }
      {
        assertion =
          !androidRequested
          || lib.elem "android-tools" config.korunix.applications;
        message = "La capacidad Android activa necesita android-tools en este host.";
      }
    ];

    # Contrato legible para el motor y la GUI. Una elección null significa
    # que Korunix todavía debe preguntarla; no se inventa una preferencia.
    environment.etc."korunix/default-roles.json".text = builtins.toJSON {
      schemaVersion = 1;
      policy = roleModel.productPolicy;
      desktopMimeFiles = roleModel.desktopMimeFileNames;
      people = map roleStateForUser cfg.users;
    };

    # KWrite y Kate comparten paquete en Nixpkgs. Solo se añade cuando alguien
    # con Plasma realmente ha elegido uno de los dos enfoques.
    environment.systemPackages = lib.optionals plasmaEditorRequested [
      (roleModel.packageFor "kate")
    ];

    # GTK4 4.20+ necesita un método de entrada para resolver correctamente
    # teclas muertas y diacríticos. Antes GNOME aportaba IBus implícitamente;
    # Korunix conserva aplicaciones GTK4 de GNOME sin conservar ese escritorio,
    # así que IBus pasa a ser el backend normal cuando no se pidió uno avanzado.
    i18n.inputMethod =
      if advancedInputMethodsEnabled
      then {
        enable = true;
        type = "fcitx5";

        fcitx5 = {
          addons = fcitxAddons;

          # GTK/Qt usan sus módulos Fcitx5. El compositor sigue siendo dueño del
          # teclado Wayland/XKB del host.
          waylandFrontend = false;
        };
      }
      else {
        enable = true;
        type = "ibus";

        ibus.waylandFrontend = true;
      };

    # Contrato interno legible para los backends de terminal y la GUI futura.
    # No contiene nombres de paquetes dentro del perfil portable de la persona.
    environment.etc."korunix/input-methods.json".text = builtins.toJSON {
      schemaVersion = 1;

      backend =
        if advancedInputMethodsEnabled
        then "fcitx5"
        else "ibus";

      launcher = "xdg-autostart";

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
