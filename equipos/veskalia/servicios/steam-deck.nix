{
  lib,
  inputs,
  equipo,
  ...
}: {
  imports = [ inputs.jovian.nixosModules.default ];

  jovian.hardware.has.amd.gpu = true;
  jovian.steam = {
    enable = true;
    autoStart = true;
    user = equipo.persona;
    desktopSession = "umbriel";
    environment = {
      STEAM_GAMESCOPE_COLOR_MANAGED = "1";
      STEAM_GAMESCOPE_VIRTUAL_WHITE = "1";
    };
  };

  # Se conserva el comportamiento de inicio que ya usa Korunix.
  services.displayManager.sddm.wayland.enable = lib.mkForce false;
  jovian.steamos.useSteamOSConfig = true;
  jovian.steamos.enableZram = false;
  services.inputplumber.enable = lib.mkForce false;
  services.power-profiles-daemon.enable = lib.mkForce false;
}
