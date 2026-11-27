{
  config,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.koru;
  preset = ./veskalia.json;
  rutaPreset = "${usuario.home}/.local/share/easyeffects/input/Korunix-Voz-Limpia.json";

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
in {
  environment.systemPackages = [pkgs.easyeffects];

  # Mantener el micrófono predeterminado.
  systemd.user.services.easyeffects = {
    description = "Easy Effects con RNNoise y puerta de ruido para Korunix";
    wants = ["pipewire.service" "wireplumber.service"];
    after = ["pipewire.service" "wireplumber.service"];

    serviceConfig = {
      Type = "simple";
      ExecStart = "${pkgs.easyeffects}/bin/easyeffects --service-mode --hide-window";
      ExecStartPost = "-${cargarPreset}";
      Restart = "on-failure";
      RestartSec = "2s";
    };
  };

  system.activationScripts.korunixEasyEffects.text = lib.mkAfter ''
    home=${lib.escapeShellArg usuario.home}
    owner="koru"
    group=${lib.escapeShellArg usuario.group}
    wants="$home/.config/systemd/user/default.target.wants"
    config_dir="$home/.config/easyeffects"
    data_dir="$home/.local/share/easyeffects"
    input_dir="$data_dir/input"

    install -d -m 0755 \
      -o "$owner" -g "$group" \
      "$wants" \
      "$config_dir" \
      "$config_dir/db" \
      "$data_dir" \
      "$input_dir"

    chown "$owner:$group" \
      "$wants" \
      "$config_dir" \
      "$config_dir/db" \
      "$data_dir" \
      "$input_dir"

    ln -sfn /etc/systemd/user/easyeffects.service \
      "$wants/easyeffects.service"
    ln -sfn ${preset} "$input_dir/Korunix-Voz-Limpia.json"

    chown -h "$owner:$group" \
      "$wants/easyeffects.service" \
      "$input_dir/Korunix-Voz-Limpia.json"
  '';
}
