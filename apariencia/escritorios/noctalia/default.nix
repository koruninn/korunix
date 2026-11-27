{
  config,
  equipo,
  inputs,
  pkgs,
  ...
}: let
  usuario = config.users.users.koru;
in {
  imports = [
    inputs.noctalia.nixosModules.default
    inputs.umbriel.nixosModules.default
    ./noctalia.nix
    ./portales.nix
    ./umbriel.nix
    ./qt.nix
  ];

  # Ocultar las herramientas de KDE en Umbriel, excepto Kdenlive.
  system.activationScripts.noctaliaDesktopEntries.text = ''
    applications_dir=${usuario.home}/.local/share/applications

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$applications_dir"

    # Quitar solo los cambios hechos por Korunix, no los personales.
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
      ${pkgs.gawk}/bin/awk '
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

      install -m 0644 \
        -o ${usuario.name} \
        -g ${usuario.group} \
        "$tmp" \
        "$applications_dir/$name"

      rm -f "$tmp"
    done
  '';
}
