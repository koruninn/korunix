{...}: {
  # Configuración propia de este equipo.
  # El hostname se obtiene automáticamente del nombre de la carpeta y las
  # funciones modernas de Nix se habilitan desde modulos/base.
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";

  # Fijar al crear la máquina según la versión de NixOS con la que nace.
  # Después no debe incrementarse simplemente por actualizar el sistema.
  system.stateVersion = "26.05";
}
