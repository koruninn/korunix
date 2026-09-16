{
  canal = "unstable";
  arquitectura = "x86_64-linux";

  # Ficha de la persona principal. Esta es la parte humana: cuenta, nombre y foto.
  persona = {
    usuario = "koru";
    nombre = "André";
    foto = ./perfil.jpg;
  };

  # Navegador predeterminado de este equipo. Chrome puede seguir instalado,
  # pero Korunix no lo selecciona automáticamente como predeterminado.
  navegadorPredeterminado = "firefox";

  # Servicio de contraseñas del escritorio. Mailspring usa este dato y ya no
  # depende del nombre de la cuenta.
  secretService = "gnome-keyring";

  # Pantalla principal de este equipo. Los compositores pueden reutilizar estos
  # datos sin incrustar detalles de hardware dentro de sus módulos.
  pantalla = {
    nombre = "DP-1";
    modo = "1920x1080@120";
    escala = 1.0;
  };
}
