{pkgs, ...}: let
  recuperarEnlaceAsix = pkgs.writeShellScript "korunix-asix-link" ''
    interfaz="$1"
    dispositivo="/sys/class/net/$interfaz"
    nmcli=${pkgs.networkmanager}/bin/nmcli
    ip=${pkgs.iproute2}/bin/ip

    [ -d "$dispositivo" ] || exit 0
    controlador="$(${pkgs.coreutils}/bin/readlink -f "$dispositivo/device/driver")"
    [ "''${controlador##*/}" = cdc_ncm ] || exit 0

    estado=""
    for intento in 1 2 3 4 5 6; do
      [ -d "$dispositivo" ] || exit 0
      estado="$(LC_ALL=C "$nmcli" -g GENERAL.STATE device show "$interfaz" 2>/dev/null || true)"
      estado="''${estado%% *}"
      case "$estado" in
        10) exit 0 ;; # No interferir con dispositivos sin gestión.
        20|30) break ;;
        40|50|60|70|80|90|100) exit 0 ;; # Ya conectando o conectado.
      esac
      ${pkgs.coreutils}/bin/sleep 1
    done
    case "$estado" in 20|30) ;; *) exit 0 ;; esac

    if ! "$ip" link set dev "$interfaz" up; then
      echo "ASIX: no se pudo activar $interfaz."
      exit 0
    fi
    ${pkgs.coreutils}/bin/sleep 3

    for intento in 1 2 3 4 5 6; do
      [ -d "$dispositivo" ] || exit 0
      ${pkgs.ethtool}/bin/ethtool "$interfaz" >/dev/null 2>&1 || true
      if [ "$(${pkgs.coreutils}/bin/cat "$dispositivo/carrier" 2>/dev/null)" = 1 ]; then
        estado="$(LC_ALL=C "$nmcli" -g GENERAL.STATE device show "$interfaz" 2>/dev/null || true)"
        estado="''${estado%% *}"
        case "$estado" in
          40|50|60|70|80|90|100) exit 0 ;;
          30)
            if "$nmcli" --wait 20 device connect "$interfaz"; then
              echo "ASIX: conexión activada en $interfaz."
            else
              echo "ASIX: NetworkManager no pudo activar $interfaz."
            fi
            exit 0
            ;;
          10) exit 0 ;;
        esac
      fi
      ${pkgs.coreutils}/bin/sleep 2
    done
    echo "ASIX: enlace no disponible en $interfaz; se mantienen los demás servicios de red."
  '';
in {
  networking.networkmanager.enable = true;

  networking.networkmanager.settings.connectivity.interval = 30;

  services.avahi = {
    enable = true;
    openFirewall = true;
  };

  hardware.bluetooth.enable = true;

  services.openssh = {
    enable = true;
    openFirewall = true;
  };

  # Acceso remoto mediante Tailscale.
  services.tailscale.enable = true;
  environment.systemPackages = with pkgs; [
    trayscale
    ethtool
    usbutils
  ];

  # Recuperar la conexión del adaptador Ethernet.
  services.udev.extraRules = ''
    ACTION=="add", SUBSYSTEM=="net", ATTRS{idVendor}=="0b95", ATTRS{idProduct}=="1790", TAG+="systemd", ENV{SYSTEMD_WANTS}+="korunix-asix-link@$name.service"
  '';

  systemd.services."korunix-asix-link@" = {
    description = "Recuperar enlace Ethernet ASIX en %I";
    after = ["NetworkManager.service"];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${recuperarEnlaceAsix} %I";
      TimeoutStartSec = "50s";
    };
  };

  # Recuperar la conexión al volver a enchufar el cable.
  systemd.services.korunix-asix-recheck = {
    description = "Comprobar la portadora del adaptador ASIX";
    after = ["NetworkManager.service"];
    serviceConfig = {
      Type = "oneshot";
      TimeoutStartSec = "10s";
    };
    script = ''
      for dispositivo in /sys/class/net/*; do
        [ -d "$dispositivo/device" ] || continue
        controlador="$(${pkgs.coreutils}/bin/readlink -f "$dispositivo/device/driver" 2>/dev/null)" || continue
        [ "''${controlador##*/}" = cdc_ncm ] || continue
        usb="$(${pkgs.coreutils}/bin/readlink -f "$dispositivo/device/.." 2>/dev/null)" || continue
        [ "$(${pkgs.coreutils}/bin/cat "$usb/idVendor" 2>/dev/null)" = 0b95 ] || continue
        [ "$(${pkgs.coreutils}/bin/cat "$usb/idProduct" 2>/dev/null)" = 1790 ] || continue
        interfaz="''${dispositivo##*/}"

        # Respetar las conexiones desactivadas a mano.
        estado="$(LC_ALL=C ${pkgs.networkmanager}/bin/nmcli -g GENERAL.STATE device show "$interfaz" 2>/dev/null || true)"
        [ "''${estado%% *}" = 20 ] || continue
        unidad="$(${pkgs.systemd}/bin/systemd-escape --template=korunix-asix-link@.service "$interfaz")"
        ${pkgs.systemd}/bin/systemctl --no-block start "$unidad"
      done
    '';
  };

  systemd.timers.korunix-asix-recheck = {
    description = "Recuperar Ethernet ASIX tras reconectar el cable";
    wantedBy = ["timers.target"];
    timerConfig = {
      OnBootSec = "5s";
      OnUnitInactiveSec = "5s";
      AccuracySec = "1s";
    };
  };

  services.sunshine = {
    enable = true;
    openFirewall = true;
    autoStart = false;
    capSysAdmin = true;
  };

  networking.firewall = rec {
    allowedTCPPorts = [
      26761
    ];

    allowedUDPPorts = [
      26761
    ];

    allowedTCPPortRanges = [
      {
        from = 1714;
        to = 1764;
      }
    ];

    allowedUDPPortRanges = allowedTCPPortRanges;
  };

  networking.firewall.enable = true;

  networking.networkmanager.settings.connectivity.enabled = true;
  networking.networkmanager.settings.connectivity.uri = "http://fedoraproject.org/static/hotspot.txt";
  networking.networkmanager.settings.connectivity.response = "OK";
}
