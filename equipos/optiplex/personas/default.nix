{
  equipo,
  ...
}: {
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = equipo.nombre or equipo.persona;
    extraGroups = ["networkmanager" "wheel"];
  };
}
