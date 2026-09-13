{
  equipo,
  ...
}: {
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = "André";
    extraGroups = [ "networkmanager" "wheel" ];
  };
}
