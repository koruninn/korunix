{pkgs, ...}: {
  # Firefox es el navegador principal de Korunix. Pywalfox queda instalado por
  # política para que Noctalia pueda empujar la paleta mediante su host nativo
  # firefox-theme, sin depender del antiguo daemon Python de pywalfox.
  programs.firefox = {
    enable = true;
    package = pkgs.firefox;

    policies.ExtensionSettings."pywalfox@frewacom.org" = {
      installation_mode = "force_installed";
      install_url = "https://addons.mozilla.org/firefox/downloads/latest/pywalfox/latest.xpi";
    };
  };
}
