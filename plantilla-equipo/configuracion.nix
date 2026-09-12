{ ... }:

{
  # Configuración propia de este equipo.
  # El hostname se obtiene automáticamente del nombre de la carpeta.

  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
}
