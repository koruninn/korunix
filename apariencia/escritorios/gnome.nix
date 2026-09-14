{
  lib,
  pkgs,
  ...
}: let
  uuid = "user-accent-colors@fabito02";

  chromaleonSource = builtins.fetchGit {
    url = "https://github.com/Fabito02/ChromaLeon.git";
    rev = "f64d1135f749ed7b6f82777f6c13e30a1cb95e6b";
  };

  chromaleon = pkgs.stdenvNoCC.mkDerivation {
    pname = "gnome-shell-extension-chromaleon";
    version = "2.3.1";
    src = chromaleonSource;

    nativeBuildInputs = [pkgs.glib];
    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      extension=$out/share/gnome-shell/extensions/${uuid}
      mkdir -p "$extension"
      cp -r . "$extension/"
      glib-compile-schemas "$extension/schemas"
    '';
  };

  gnomeDynamicColors = pkgs.writeShellApplication {
    name = "korunix-gnome-colors";
    runtimeInputs = [
      pkgs.glib
      pkgs.gnome-shell
    ];
    text = ''
      extension_dir="${chromaleon}/share/gnome-shell/extensions/${uuid}"

      # GNOME recibe su propia capa GTK y desconecta tanto Noctalia como la
      # colors.css que Plasma genera con kde-gtk-config.
      korunix-gtk-session gnome

      export GSETTINGS_SCHEMA_DIR="$extension_dir/schemas"

      gsettings set org.gnome.shell.extensions.chromaleon gnome-colors false
      gsettings set org.gnome.shell.extensions.chromaleon custom-color false
      gsettings set org.gnome.shell.extensions.chromaleon tint-shell true
      gsettings set org.gnome.shell.extensions.chromaleon tint-panel true
      gsettings set org.gnome.shell.extensions.chromaleon tint-apps true
      gsettings set org.gnome.shell.extensions.chromaleon tint-gtk3 true
      gsettings set org.gnome.shell.extensions.chromaleon tinting-strength 1
      gsettings set org.gnome.shell.extensions.chromaleon persistent-choices false
      gsettings set org.gnome.shell.extensions.chromaleon recolor-folders true
      gsettings set org.gnome.shell.extensions.chromaleon recolor-apps false

      gnome-extensions enable ${uuid} >/dev/null 2>&1 || true
    '';
  };
in {
  services.desktopManager.gnome.enable = true;
  services.desktopManager.gnome.sessionPath = [chromaleon];

  # Plasma y GNOME definen sendos askpass con la misma prioridad. Seahorse se
  # usa como implementación común mientras ambas sesiones conviven.
  programs.ssh.askPassword =
    lib.mkForce "${pkgs.seahorse}/libexec/seahorse/ssh-askpass";

  environment.systemPackages = [gnomeDynamicColors];

  environment.etc."xdg/autostart/korunix-gnome-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=Korunix GNOME Colors
Comment=Aplica colores dinámicos de ChromaLeon a GNOME y GTK
Exec=${gnomeDynamicColors}/bin/korunix-gnome-colors
OnlyShowIn=GNOME;
NoDisplay=true
X-GNOME-Autostart-enabled=true
  '';
}
