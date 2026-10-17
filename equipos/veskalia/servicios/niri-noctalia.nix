{ lib, pkgs, ... }:
{
  imports = [
    ../../../apariencia/escritorios/noctalia
  ];

  # Niri se registra como sesión independiente; su configuración está en Git.
  programs.niri.enable = true;
  environment.etc."niri/config.kdl".source = ./niri-config.kdl;

  # Programas utilizados por los atajos declarados en niri-config.kdl.
  environment.systemPackages = [
    pkgs.brightnessctl
    pkgs.fuzzel
    pkgs.kdePackages.konsole
    pkgs.playerctl
  ];

  # Noctalia se ejecuta en la sesión Niri, no en Plasma.
  programs.noctalia.systemd = {
    enable = true;
    target = "niri.service";
  };

  # Tras salir de una sesión, permitir elegir entre Plasma, Niri o Gaming Mode.
  # El inicio automático inicial de Gaming Mode se mantiene.
}
