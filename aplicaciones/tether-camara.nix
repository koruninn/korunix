{config, lib, pkgs, ...}: let
  usuario = config.users.users.koru;
  nombre = "iPhone (Tether)";
  dispositivo = "/dev/video20";
  sysfs = "/sys/class/video4linux/video20";
  loopback = config.boot.kernelPackages.v4l2loopback;
  gst = pkgs.gst_all_1;

  crearCamara = pkgs.writeShellApplication {
    name = "korunix-crear-camara-iphone";
    runtimeInputs = [pkgs.kmod pkgs.coreutils pkgs.systemd loopback.bin];
    text = ''
      modprobe v4l2loopback
      if [ ! -e ${sysfs} ] && [ ! -e ${dispositivo} ]; then
        v4l2loopback-ctl add -n ${lib.escapeShellArg nombre} -x 1 ${dispositivo}
      fi
      if [ "$(cat ${sysfs}/name 2>/dev/null)" != ${lib.escapeShellArg nombre} ] ||
         [ "$(readlink -f ${sysfs})" != /sys/devices/virtual/video4linux/video20 ]; then
        echo "El número de cámara está ocupado; no se modificará ningún dispositivo." >&2
        exit 1
      fi
      udevadm trigger --action=change --settle ${sysfs}
      test -c ${dispositivo}
    '';
  };

  transmitir = pkgs.writeShellApplication {
    name = "korunix-camara-iphone";
    runtimeInputs = [pkgs.pipewire pkgs.jq pkgs.coreutils gst.gstreamer pkgs.ffmpeg];
    text = ''
      export GST_PLUGIN_PATH_1_0=${lib.makeSearchPath "lib/gstreamer-1.0" [
        gst.gstreamer.out
        pkgs.pipewire
        gst.gst-plugins-base
        gst.gst-plugins-good
      ]}
      export GST_PLUGIN_PATH="$GST_PLUGIN_PATH_1_0"

      # Comprobar los elementos antes de esperar la cámara.
      for elemento in pipewiresrc capsfilter queue y4menc fdsink; do
        if ! gst-inspect-1.0 "$elemento" >/dev/null 2>&1; then
          echo "Falta el complemento de GStreamer: $elemento" >&2
          exit 78
        fi
      done

      buscar_camara() {
        [ -w ${dispositivo} ] &&
          [ "$(cat ${sysfs}/name 2>/dev/null)" = ${lib.escapeShellArg nombre} ] &&
          timeout 5 pw-dump | jq -er '
            [.[] | select(.type == "PipeWire:Interface:Node" and
              .info.props["node.name"] == "tether-camera" and
              .info.props["media.class"] == "Video/Source") |
              .info.props["object.serial"] | select(type == "number")] |
            max // empty
          ' 2>/dev/null
      }

      # El identificador cambia cuando Tether vuelve a crear la cámara.
      until serial="$(buscar_camara)"; do
        sleep 3
      done

      echo "Conectando la cámara del iPhone (PipeWire $serial)."
      # FFmpeg evita que la imagen se congele al pasar a V4L2.
      (
        gst-launch-1.0 -q pipewiresrc target-object="$serial" \
          ! video/x-raw,format=I420 ! queue ! y4menc ! fdsink fd=1 sync=false |
        ffmpeg -hide_banner -loglevel warning -nostdin -f yuv4mpegpipe -i pipe:0 \
          -c:v rawvideo -pix_fmt yuv420p -f v4l2 ${dispositivo}
      ) &
      pid=$!

      detener() {
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
      }
      trap detener EXIT
      trap 'exit 0' TERM INT

      # Volver a conectar si desaparece la fuente de PipeWire.
      while kill -0 "$pid" 2>/dev/null && [ "$(buscar_camara)" = "$serial" ]; do
        sleep 3
      done
    '';
  };
in lib.mkIf config.programs.tether.enable {
  boot.extraModulePackages = [loopback];
  boot.kernelModules = ["v4l2loopback"];

  # Esta cámara es independiente de la cámara virtual de OBS.
  services.udev.extraRules = ''
    SUBSYSTEM=="video4linux", KERNEL=="video20", ATTR{name}=="iPhone (Tether)", OWNER="${usuario.name}", MODE="0600"
  '';

  systemd.services.korunix-camara-iphone = {
    description = "Crear la cámara virtual del iPhone";
    wantedBy = ["multi-user.target"];
    after = ["systemd-modules-load.service" "systemd-udevd.service"];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = "${crearCamara}/bin/korunix-crear-camara-iphone";
    };
  };

  systemd.user.services.korunix-camara-iphone = {
    description = "Transmitir la cámara del iPhone a las aplicaciones";
    wantedBy = ["default.target"];
    after = ["pipewire.service" "tetherd.service"];
    wants = ["pipewire.service"];
    unitConfig.ConditionUser = usuario.name;
    serviceConfig = {
      ExecStart = "${transmitir}/bin/korunix-camara-iphone";
      KillMode = "control-group";
      Restart = "always";
      RestartSec = 5;
      RestartPreventExitStatus = "78";
    };
  };

  environment.systemPackages = [pkgs.v4l-utils];
}
