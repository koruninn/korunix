{
  equipo,
  pkgs,
  ...
}: {
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";
  services.printing.enable = true;

  # Niri reutiliza la pantalla declarada por el equipo. Así monitor, modo y
  # escala tienen una sola fuente de verdad compartida con los otros compositores.
  environment.etc."niri/monitor.kdl".text = ''
    output "${equipo.pantalla.nombre}" {
        mode "${equipo.pantalla.modo}"
        scale ${toString equipo.pantalla.escala}
    }
  '';

  nixpkgs.config.allowUnfree = true;
  environment.systemPackages = [pkgs.alejandra];

  system.stateVersion = "26.05";
}
