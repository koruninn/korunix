{
  equipo,
  ...
}: {
  # Niri incluye siempre este archivo. Si el equipo declara una pantalla,
  # traducimos aquí sus datos al formato de Niri; si no, dejamos que el
  # compositor detecte la salida y sus valores automáticamente.
  environment.etc."niri/monitor.kdl".text =
    if equipo ? pantalla
    then ''
      output "${equipo.pantalla.nombre}" {
          mode "${equipo.pantalla.modo}"
          scale ${toString equipo.pantalla.escala}
      }
    ''
    else ''
      // Sin configuración fija de monitor: Niri usa detección automática.
    '';
}
