{...}: {
  services.power-profiles-daemon.enable = true;
  services.upower.enable = true;

  # Una pulsación del botón de encendido suspende el equipo.
  services.logind.settings.Login = {
    HandlePowerKey = "suspend";
    HandleSuspendKey = "suspend";
  };
}
