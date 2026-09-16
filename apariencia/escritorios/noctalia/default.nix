{
  config,
  equipo,
  inputs,
  pkgs,
  ...
}: let
  filtrarAplicaciones = pkgs.writeShellApplication {
    name = "korunix-noctalia-desktop-entries";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.gawk
      pkgs.gnugrep
    ];
    text = ''
      applications_dir="''${XDG_DATA_HOME:-$HOME/.local/share}/applications"
      install -d -m 0755 "$applications_dir"

      # Elimina únicamente overrides creados por Korunix para no tocar cambios
      # manuales de la persona usuaria.
      for desktop in \
        "$applications_dir"/org.kde.*.desktop \
        "$applications_dir"/systemsettings.desktop
      do
        [ -e "$desktop" ] || continue
        if grep -q '^X-Korunix-Noctalia-Hidden=true$' "$desktop"; then
          rm -f "$desktop"
        fi
      done

      for desktop in \
        ${config.system.path}/share/applications/org.kde.*.desktop \
        ${config.system.path}/share/applications/systemsettings.desktop
      do
        [ -e "$desktop" ] || continue

        name=$(basename "$desktop")
        [ "$name" = "org.kde.kdenlive.desktop" ] && continue

        tmp=$(mktemp)
        awk '
          {
            lines[NR] = $0
            if ($0 ~ /^NotShowIn=/) {
              value = substr($0, 11)
              sub(/;*$/, "", value)
              if (value !~ /(^|;)umbriel(;|$)/)
                value = value ";umbriel"
              lines[NR] = "NotShowIn=" value ";"
              has_not_show_in = 1
            }
          }
          END {
            for (i = 1; i <= NR; ++i) {
              print lines[i]
              if (lines[i] == "[Desktop Entry]") {
                print "X-Korunix-Noctalia-Hidden=true"
                if (!has_not_show_in)
                  print "NotShowIn=umbriel;"
              }
            }
          }
        ' "$desktop" > "$tmp"

        install -m 0644 "$tmp" "$applications_dir/$name"
        rm -f "$tmp"
      done
    '';
  };
in {
  imports = [
    inputs.noctalia.nixosModules.default
    ./noctalia.nix
    ./fondos-dia-noche.nix
    ./portales.nix
    ./umbriel.nix
    ./qt.nix
  ];

  # Las aplicaciones KDE siguen instaladas cuando alguna aplicación las necesita,
  # pero Noctalia no las muestra en Umbriel. Kdenlive es la única excepción visible.
  systemd.user.services.korunix-noctalia-desktop-entries = {
    description = "Oculta aplicaciones KDE auxiliares en Umbriel";
    wantedBy = ["default.target"];
    unitConfig.ConditionUser = equipo.persona;
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${filtrarAplicaciones}/bin/korunix-noctalia-desktop-entries";
    };
  };
}
