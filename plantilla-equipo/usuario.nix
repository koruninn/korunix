{...}: {
  users.users.usuario = {
    isNormalUser = true;
    description = "Usuario";
    extraGroups = ["networkmanager" "wheel"];
  };
}
