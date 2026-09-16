{
  # Canal de NixOS: "stable" o "unstable".
  canal = "stable";

  # Arquitectura de la máquina.
  arquitectura = "x86_64-linux";

  # Persona principal del equipo. Solo usuario es obligatorio.
  persona = {
    usuario = "usuario";
    # nombre = "Nombre visible";
    # foto = ./perfil.jpg;
  };

  # Opcionales. Añádelos solo si ese equipo los necesita:
  # navegadorPredeterminado = "firefox";
  # secretService = "gnome-keyring";
  # pantalla = {
  #   nombre = "DP-1";
  #   modo = "1920x1080@120";
  #   escala = 1.0;
  # };
}
