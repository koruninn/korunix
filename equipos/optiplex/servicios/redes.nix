{...}: {
  # El nombre de este equipo sale solo del nombre de la carpeta donde está la ajustes.
  networking.networkmanager.enable = true;

  # Avahi deja ver este equipo y otros servicios compatibles dentro de la red local.
  services.avahi = {
    enable = true;
    openFirewall = true;
  };

  # SSH deja entrar a este equipo desde otro aparato de forma remota.
  services.openssh = {
    enable = true;
    openFirewall = true;
  };

  # LocalSend abre su propio puerto cuando hace falta.
  # No dejamos abiertos puertos adicionales para servicios que este equipo no utiliza.
  networking.firewall.enable = true;
}
