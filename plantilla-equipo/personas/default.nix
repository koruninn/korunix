{
  equipo,
  ...
}: {
  # Persona principal del equipo. Cambia "usuario" en equipo.nix por el nombre
  # de inicio de sesión que quieras usar; los módulos reutilizables lo tomarán
  # de equipo.persona automáticamente.
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = "Usuario";
    extraGroups = [ "networkmanager" "wheel" ];
  };
}
