{...}: {
  # Conserva el almacenamiento cifrado de contraseñas y secretos.
  services.gnome.gnome-keyring.enable = true;

  # Integración PAM para las sesiones de Jovian y SDDM.
  security.pam.services.sddm.enableGnomeKeyring = true;

  # Se conserva el soporte anterior por si greetd vuelve a utilizarse.
  security.pam.services.greetd.enableGnomeKeyring = true;
}
