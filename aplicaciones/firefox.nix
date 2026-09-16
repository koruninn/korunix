{
  equipo,
  pkgs,
  ...
}: let
  pywalfoxManifest = pkgs.writeText "pywalfox-native-messaging-host.json" (
    builtins.toJSON {
      name = "pywalfox";
      description = "Pywalfox native messaging host";
      path = "${pkgs.pywalfox-native}/bin/pywalfox";
      type = "stdio";
      allowed_extensions = ["pywalfox@frewacom.org"];
    }
  );

  registrarPywalfox = pkgs.writeShellApplication {
    name = "korunix-pywalfox-native-host";
    runtimeInputs = [pkgs.coreutils];
    text = ''
      manifest_dir="$HOME/.mozilla/native-messaging-hosts"
      install -d -m 0755 "$manifest_dir"
      install -m 0644 ${pywalfoxManifest} "$manifest_dir/pywalfox.json"
    '';
  };
in {
  # Firefox es el navegador principal de Korunix. La extensión Pywalfox se
  # instala por política y el host nativo se registra declarativamente para
  # que la plantilla comunitaria `pywalfox` de Noctalia pueda actualizarlo.
  programs.firefox = {
    enable = true;
    package = pkgs.firefox;

    policies.ExtensionSettings."pywalfox@frewacom.org" = {
      installation_mode = "force_installed";
      install_url = "https://addons.mozilla.org/firefox/downloads/latest/pywalfox/latest.xpi";
    };
  };

  systemd.user.services.korunix-pywalfox-native-host = {
    description = "Registra Pywalfox para Firefox";
    wantedBy = ["default.target"];
    unitConfig.ConditionUser = equipo.persona;
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${registrarPywalfox}/bin/korunix-pywalfox-native-host";
    };
  };
}
