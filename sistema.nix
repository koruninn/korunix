{
  aplicaciones,
  escritorio,
  lib,
  nombre,
  personas,
  pkgs,
  programa,
  ...
}: let
  cuentas = map (persona: persona.cuenta) personas;

  personasValidas =
    if personas == []
    then throw "Falta al menos una cuenta en [[personas]]."
    else if builtins.length cuentas != builtins.length (lib.unique cuentas)
    then throw "Hay una cuenta repetida en [[personas]]."
    else personas;

  usuarios = builtins.listToAttrs (map (persona: {
      name = persona.cuenta;
      value = {
        isNormalUser = true;
        description = persona.nombre;
        shell = pkgs.fish;
        extraGroups =
          ["networkmanager"]
          ++ lib.optionals (persona.administrador or false) ["wheel"];
      };
    })
    personasValidas);

  sesion =
    if escritorio == "niri"
    then "niri"
    else if escritorio == "hyprland"
    then "hyprland-uwsm"
    else if escritorio == "cinnamon"
    then "cinnamon-wayland"
    else if escritorio == "plasma"
    then "plasma"
    else throw "No conozco el escritorio «${escritorio}».";
in {
  imports = [./hardware.nix];

  networking.hostName = nombre;
  networking.networkmanager.enable = true;

  nix.settings.experimental-features = ["nix-command" "flakes"];
  nixpkgs.config.allowUnfree = true;

  programs.fish.enable = true;

  # La contraseña no se guarda aquí. La cuenta que ya existe conserva la suya.
  users.mutableUsers = true;
  users.users = usuarios;

  # GDM muestra las sesiones; GNOME no se instala como escritorio.
  services.xserver.enable = true;
  services.displayManager = {
    gdm.enable = true;
    defaultSession = sesion;
  };

  programs.niri.enable = escritorio == "niri";

  programs.hyprland = lib.mkIf (escritorio == "hyprland") {
    enable = true;
    withUWSM = true;
    xwayland.enable = true;
  };

  services.xserver.desktopManager.cinnamon.enable = escritorio == "cinnamon";
  services.desktopManager.plasma6.enable = escritorio == "plasma";

  environment.systemPackages = aplicaciones ++ [programa];

  # Conserva la compatibilidad de la instalación actual.
  system.stateVersion = "26.05";
}
