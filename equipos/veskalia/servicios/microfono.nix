{ config, equipo, lib, pkgs, ... }:
let
  usuario = config.users.users.${equipo.persona};
  preset = ./easyeffects-voz-limpia.json;
  rutaPreset = "${usuario.home}/.config/easyeffects/input/Korunix-Voz-Limpia.json";

  cargarPreset = pkgs.writeShellScript "korunix-easyeffects-load-preset" ''
    ${pkgs.coreutils}/bin/sleep 2
    for intento in 1 2 3 4 5; do
      if ${pkgs.easyeffects}/bin/easyeffects --load-preset Korunix-Voz-Limpia; then
        exit 0
      fi
      ${pkgs.coreutils}/bin/sleep 1
    done
    exit 1
  '';
in
{
  environment.systemPackages = [ pkgs.easyeffects ];

  # EasyEffects se inicia en la sesión del usuario; no requiere Home Manager.
  # El micrófono físico permanece como dispositivo predeterminado.
  systemd.user.services.easyeffects = {
    description = "Easy Effects con RNNoise y puerta de ruido para Korunix";
    wants = [ "pipewire.service" "wireplumber.service" ];
    after = [ "pipewire.service" "wireplumber.service" ];

    serviceConfig = {
      Type = "simple";
      ExecStart = "${pkgs.easyeffects}/bin/easyeffects --service-mode --hide-window";
      ExecStartPost = "-${cargarPreset}";
      Restart = "on-failure";
      RestartSec = "2s";
    };
  };

  # Instala el preset gestionado por Nix en el perfil de la persona de Veskalia
  # y habilita el servicio solo para esa cuenta, sin crear filtros PipeWire propios.
  system.activationScripts.korunixEasyEffects.text = lib.mkAfter ''
    install -d -m 0755 "${usuario.home}/.config/systemd/user/default.target.wants"
    install -d -m 0755 "${usuario.home}/.config/easyeffects/input"
    chown ${equipo.persona}:${usuario.group} \
      "${usuario.home}/.config/systemd/user/default.target.wants" \
      "${usuario.home}/.config/easyeffects/input"

    ln -sfn /etc/systemd/user/easyeffects.service \
      "${usuario.home}/.config/systemd/user/default.target.wants/easyeffects.service"
    ln -sfn ${preset} "${rutaPreset}"

    chown -h ${equipo.persona}:${usuario.group} \
      "${usuario.home}/.config/systemd/user/default.target.wants/easyeffects.service" \
      "${rutaPreset}"
  '';
}
