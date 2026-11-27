{
  config,
  equipo,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.koru;
  script = pkgs.writeText "99-night-mode-test.lua" ''
    gamescope.hook("OnPostPaint", function()
      gamescope.convars.composite_force.value = true
    end)
  '';
in {
  # Permitir que Steam ajuste el brillo sin dar acceso a otros usuarios.
  users.groups.korunix-backlight = {};
  users.users.koru.extraGroups = ["korunix-backlight"];
  services.udev.extraRules = ''
    ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="amdgpu_bl*", RUN+="${pkgs.coreutils}/bin/chgrp korunix-backlight /sys%p/brightness", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys%p/brightness"
  '';
  environment.systemPackages = [pkgs.brightnessctl];

  # Activar la prueba solo cuando se solicite; quitar únicamente el enlace de Korunix.
  environment.etc."korunix/gamescope/99-night-mode-test.lua" =
    lib.mkIf (equipo.gamescopeForzarComposicion or false) {source = script;};

  system.activationScripts.korunixGamescopeNightMode.text = ''
    home=${lib.escapeShellArg usuario.home}
    owner=${lib.escapeShellArg usuario.name}
    group=${lib.escapeShellArg usuario.group}
    destino="$home/.config/gamescope/scripts/99-night-mode-test.lua"

    ${if (equipo.gamescopeForzarComposicion or false) then ''
      install -d -m 0755 -o "$owner" -g "$group" "$home/.config/gamescope/scripts"
      if [ ! -e "$destino" ] && [ ! -L "$destino" ]; then
        ln -s ${script} "$destino"
        chown -h "$owner:$group" "$destino"
      fi
    '' else ''
      if [ -L "$destino" ]; then
        case "$(readlink "$destino")" in
          /nix/store/*-99-night-mode-test.lua)
            rm -- "$destino"
            ;;
        esac
      fi
    ''}
  '';
}
