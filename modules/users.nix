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

  accountNameFor = userId: profiles.${userId}.accountName or userId;

  groupsFor = profile:
    lib.unique (
      ["networkmanager"]
      ++ lib.optionals (profile.administrator or false) ["wheel"]
      ++ lib.optionals (lib.elem "virtualization" (profile.capabilities or [])) [
        "libvirtd"
        "kvm"
      ]
      ++ lib.optionals (lib.elem "sunshine" (profile.capabilities or [])) [
        "input"
        "uinput"
      ]
      ++ lib.optionals (lib.elem "printing" (profile.capabilities or [])) [
        "lp"
        "scanner"
      ]
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
          home = profile.homeDirectory;
          shell = pkgs.fish;
          extraGroups = groupsFor profile;
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
        language = profile.language or config.korunix.localization.language;
      in ''
        ${lib.escapeShellArg accountName})
          KORUNIX_USER_ID=${lib.escapeShellArg userId}
          KORUNIX_LANGUAGE=${lib.escapeShellArg language}
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
    # mutableUsers conserva las contraseñas creadas por Calamares. Korunix declara
    # identidad y capacidades, pero no coloca hashes de contraseñas en Git.
    users.mutableUsers = true;
    users.users = usersConfig;


    systemd.user.services.korunix-user-prepare = {
      description = "Prepara los archivos personales administrados por Korunix";

      wantedBy = [
        "graphical-session.target"
      ];

      before = [
        "noctalia.service"
      ];

      path = [
        pkgs.coreutils
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
