{pkgs, ...}: {
  programs.firefox = {
    enable = true;
    package = pkgs.firefox;
    # Noctalia incorpora el host nativo; solo se instala el complemento de Firefox.
    policies.ExtensionSettings."pywalfox@frewacom.org" = {
      installation_mode = "normal_installed";
      install_url = "https://addons.mozilla.org/firefox/downloads/latest/pywalfox/latest.xpi";
    };
  };
}
