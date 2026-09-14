{pkgs, ...}: {
  # Firefox es el navegador principal de Korunix. La extensión Pywalfox se
  # instala por política. Con la plantilla pywalfox-beta4, Noctalia es el
  # native-messaging host; no se instala pywalfox-native porque el host externo
  # puede competir con el manifest que administra Noctalia.
  programs.firefox = {
    enable = true;
    package = pkgs.firefox;

    policies.ExtensionSettings."pywalfox@frewacom.org" = {
      installation_mode = "force_installed";
      install_url = "https://addons.mozilla.org/firefox/downloads/latest/pywalfox/latest.xpi";
    };
  };
}
