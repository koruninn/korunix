{
  canal = "unstable";
  arquitectura = "x86_64-linux";

  # Persona principal del equipo. Los módulos reutilizables usan este nombre
  # en vez de asumir un usuario concreto.
  persona = "koru";

  # Navegador predeterminado de este equipo. Otros equipos conservan Chrome
  # salvo que declaren explícitamente otra opción.
  navegadorPredeterminado = "zen";

  # Pantalla principal de este equipo. Los compositores pueden reutilizar estos
  # datos sin incrustar detalles de hardware dentro de sus módulos.
  pantalla = {
    nombre = "DP-1";
    modo = "1920x1080@120";
    escala = 1.0;
  };
}
