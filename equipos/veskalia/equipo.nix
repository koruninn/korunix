{
  canal = "unstable";
  arquitectura = "x86_64-linux";

  # Persona principal del equipo. Los bloques reutilizables usan este nombre
  # en vez de asumir un persona concreto.
  persona = "koru";

  # Navegador predeterminado de este equipo. Chrome puede seguir instalado,
  # pero Korunix no lo selecciona automáticamente como predeterminado.
  navegadorPredeterminado = "firefox";

  # Pantalla principal de este equipo. Los compositores pueden reutilizar estos
  # datos sin incrustar detalles de hardware dentro de sus bloques.
  pantalla = {
    nombre = "DP-1";
    modo = "1920x1080@120";
    escala = 1.0;
  };
}
