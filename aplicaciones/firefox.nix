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
  # Firefox es el navegador principal de Korunix.
  # La extensión Pywalfox se instala de forma automática para que el estilo
  # visual del sistema se mantenga igual al navegar.
  # Así la persona no tiene que instalar nada a mano ni repetir pasos.
  programs.firefox = {
    enable = true;
    package = pkgs.firefox;

    policies.ExtensionSettings."pywalfox@frewacom.org" = {
      installation_mode = "force_installed";
      install_url = "https://addons.mozilla.org/firefox/downloads/latest/pywalfox/latest.xpi";
    };
  };

  # Aquí se crea el archivo que le dice a Firefox qué extensión puede usar.
  # Esto evita que la persona tenga que correr comandos a mano después de
  # reinstalar o limpiar el sistema.
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
