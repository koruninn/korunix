{
  # Indica si este equipo debe usar la versión estable o la versión de desarrollo de NixOS.
  canal = "stable";

  # Señala la arquitectura del procesador y del equipo que tendrá este equipo.
  arquitectura = "x86_64-linux";

  # Es el nombre de la cuenta principal que se creará en este equipo.
  persona = "usuario";

  # Estos datos son opcionales. Solo agrega lo que este equipo realmente necesite.
  # navegadorPredeterminado = "firefox"; # Navegador que se abrirá por defecto.
  # secretService = "gnome-keyring"; # Almacén que guardará contraseñas y otros secretos.
  # pantalla = {
  # nombre = "DP-1"; # Nombre que usa el equipo para identificar la pantalla.
  # modo = "1920x1080@120"; # Resolución y frecuencia de actualización de la pantalla.
  # escala = 1.0; # Tamaño con el que se mostrarán ventanas y textos.
  # };
}
