{config, lib, pkgs, ...}: let
  usuario = config.users.users.koru;
in {
  # Steam puede ajustar el brillo de la pantalla.
  users.groups.korunix-backlight = {};
  users.users.koru.extraGroups = ["korunix-backlight"];
  services.udev.extraRules = ''
    ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="amdgpu_bl*", RUN+="${pkgs.coreutils}/bin/chgrp korunix-backlight /sys%p/brightness", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys%p/brightness"
  '';
  environment.systemPackages = [pkgs.brightnessctl];

  system.activationScripts.korunixGamescopeNightMode.text = ''
    destino=${lib.escapeShellArg "${usuario.home}/.config/gamescope/scripts/99-night-mode-test.lua"}
    if [ -L "$destino" ]; then
      case "$(readlink "$destino")" in
        /nix/store/*-99-night-mode-test.lua) rm -- "$destino" ;;
      esac
    fi
  '';
}
