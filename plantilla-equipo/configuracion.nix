{...}: {
  # Configuración propia de este equipo.
  # El hostname se obtiene automáticamente del nombre de la carpeta y las
  # funciones modernas de Nix se habilitan desde modulos/base.
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";
}
