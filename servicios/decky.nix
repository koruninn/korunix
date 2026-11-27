{config, equipo, lib, pkgs, ...}: let
  usuario = config.users.users.koru;
  decky = pkgs.callPackage ./decky-paquete.nix {};
  estado = "/var/lib/decky-loader";
  catalogo = pkgs.writeText "decky-complementos.json" (builtins.toJSON (import ./decky-complementos.nix));
  temasSteam = "${usuario.home}/.local/share/decky-themes";
  prepararTemas = pkgs.writeShellScript "korunix-decky-themes" ''
    set -eu
    steamui=${lib.escapeShellArg "${usuario.home}/.local/share/Steam/steamui"}
    destino=${lib.escapeShellArg temasSteam}
    enlace="$steamui/themes_custom"

    mkdir -p "$steamui"
    if [ -L "$enlace" ]; then
      actual="$(readlink "$enlace")"
      if [ "$actual" = "$destino" ]; then
        exit 0
      fi
      if [ "$actual" != "${estado}/themes" ]; then
        echo "Korunix: se conserva el enlace personalizado de CSS Loader: $actual" >&2
        exit 0
      fi
    elif [ -e "$enlace" ]; then
      echo "Korunix: se conserva el directorio themes_custom existente." >&2
      exit 0
    fi
    ln -sfnT "$destino" "$enlace"
  '';
in {
  # Mantener el servicio que esperan los complementos de Decky.
  environment.systemPackages = [decky];
  systemd.tmpfiles.rules = [
    "d ${estado} 0755 ${usuario.name} ${usuario.group} -"
    "d ${estado}/themes 0755 ${usuario.name} ${usuario.group} -"
    "d ${temasSteam} 0755 ${usuario.name} ${usuario.group} -"
  ];

  # Compartir los temas con Steam sin mover ni duplicar archivos.
  systemd.mounts = [{
    what = "${estado}/themes";
    where = temasSteam;
    type = "none";
    options = "bind";
    wantedBy = ["multi-user.target"];
    # Crear los directorios antes de montar los temas para evitar un ciclo al arrancar.
    unitConfig.DefaultDependencies = false;
    requires = ["systemd-tmpfiles-setup.service"];
    after = ["local-fs.target" "systemd-tmpfiles-setup.service"];
    before = ["umount.target"];
    conflicts = ["umount.target"];
  }];

  # Habilitar la depuración de la interfaz de Steam que necesita Decky.
  system.activationScripts.korunixDeckyCEF = {
    deps = ["korunixSteamPath"];
    text = ''
      home=${lib.escapeShellArg usuario.home}
      owner=${lib.escapeShellArg usuario.name}
      group=${lib.escapeShellArg usuario.group}
      install -d -m 0755 -o "$owner" -g "$group" "$home/.local/share/Steam"
      ${pkgs.util-linux}/bin/runuser -u "$owner" -- ${pkgs.coreutils}/bin/touch "$home/.local/share/Steam/.cef-enable-remote-debugging"
    '';
  };

  systemd.services.korunix-decky-complementos = {
    description = "Recuperar los complementos y temas de Decky";
    before = ["plugin_loader.service"];
    after = ["network-online.target" "systemd-tmpfiles-setup.service"];
    wants = ["network-online.target"];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = "${pkgs.python3}/bin/python3 ${./decky-complementos.py} ${catalogo} ${estado} ${usuario.name}";
      TimeoutStartSec = "10min";
      Environment = "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
    };
  };

  systemd.services.plugin_loader = {
    description = "Decky Loader";
    wantedBy = ["multi-user.target"];
    after = ["network.target" "systemd-tmpfiles-setup.service" "korunix-decky-complementos.service"];
    wants = ["korunix-decky-complementos.service"];
    unitConfig.RequiresMountsFor = temasSteam;
    preStart = ''
      ${pkgs.util-linux}/bin/runuser -u ${lib.escapeShellArg usuario.name} -- ${prepararTemas}
    '';
    environment = {
      UNPRIVILEGED_USER = usuario.name;
      UNPRIVILEGED_PATH = estado;
      PRIVILEGED_PATH = estado;
      PLUGIN_PATH = "${estado}/plugins";
      LOG_LEVEL = "INFO";
    };
    path = with pkgs; [coreutils psmisc procps curl unzip gnutar gzip util-linux systemd iproute2];
    serviceConfig = {
      User = "root";
      ExecStart = "${decky}/bin/decky-loader";
      WorkingDirectory = estado;
      Restart = "on-failure";
      RestartSec = 5;
      KillMode = "process";
      TimeoutStopSec = 45;
    };
  };
}
