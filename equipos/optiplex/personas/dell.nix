{ ... }: {
  users.users.dell = {
    isNormalUser = true;
    description = "dell";
    extraGroups = [ "networkmanager" "wheel" ];
  };
}
