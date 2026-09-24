{ ... }:
{
  # Korunix usa GNOME Keyring como Secret Service para aplicaciones como
  # Mailspring. Se limita a este equipo para no alterar el comportamiento de
  # Optiplex.
  services.gnome.gnome-keyring.enable = true;

  # Noctalia Greeter inicia la sesión mediante greetd; PAM desbloquea el
  # keyring con la misma contraseña del inicio de sesión.
  security.pam.services.greetd.enableGnomeKeyring = true;
}
