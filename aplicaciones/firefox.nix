{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  pywalfoxManifest = pkgs.writeText "pywalfox-native-messaging-host.json" (
    builtins.toJSON {
      name = "pywalfox";
      description = "Pywalfox native messaging host";
      path = "${pkgs.pywalfox-native}/bin/pywalfox";
      type = "stdio";
      allowed_extensions = ["pywalfox@frewacom.org"];
    }
  );
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

  # Instalar el manifest por usuario evita depender de ejecutar manualmente
  # `pywalfox install` después de cada instalación limpia de Korunix.
  system.activationScripts.pywalfoxNativeHost.text = ''
    manifest_dir=${usuario.home}/.mozilla/native-messaging-hosts

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$manifest_dir"

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${pywalfoxManifest} \
      "$manifest_dir/pywalfox.json"
  '';
}
