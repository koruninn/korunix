{
  equipo,
  ...
}: {
  # Persona principal del equipo. Cambia "persona" en equipo.nix por el nombre
  # de inicio de sesión que quieras usar; los bloques reutilizables lo tomarán
  # de equipo.persona automáticamente.
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = "Usuario";
    extraGroups = [ "networkmanager" "wheel" ];
  };
}
