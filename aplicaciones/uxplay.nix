{pkgs, ...}: let
  airplay = pkgs.writeShellScriptBin "korunix-airplay" ''
    exec ${pkgs.uxplay}/bin/uxplay -n Veskalia -nh -p 35000 -avdec -vsync no
  '';

  lanzador = pkgs.makeDesktopItem {
    name = "korunix-airplay";
    desktopName = "UxPlay";
    genericName = "Duplicar pantalla del iPhone";
    comment = "Recibir la pantalla y el audio del iPhone";
    exec = "${airplay}/bin/korunix-airplay";
    icon = "video-display";
    terminal = true;
    categories = ["AudioVideo" "Network"];
  };
in {
  environment.systemPackages = [
    pkgs.uxplay
    airplay
    lanzador
  ];

  # AirPlay usa estos tres puertos para recibir imagen y sonido.
  networking.firewall.allowedTCPPorts = [35000 35001 35002];
  networking.firewall.allowedUDPPorts = [35000 35001 35002];
}
