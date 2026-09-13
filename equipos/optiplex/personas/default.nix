{
  equipo,
  ...
}: {
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = equipo.persona;
    extraGroups = ["networkmanager" "wheel"];
  };
}
