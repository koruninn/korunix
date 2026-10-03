{ ... }:
{
  # GNOME Keyring guarda en forma segura las contraseñas y secretos que usan apps como Mailspring.
  # Se activa solo en este equipo para no cambiar lo que hace el Optiplex.
  services.gnome.gnome-keyring.enable = true;

  # Al iniciar sesión con Noctalia Greeter, el equipo intenta desbloquear automáticamente
  # el almacén de contraseñas usando la misma contraseña del inicio de sesión.
  security.pam.services.greetd.enableGnomeKeyring = true;
}
