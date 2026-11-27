{...}: {
  users.users.koru = {
    isNormalUser = true;
    description = "André";
    extraGroups = ["networkmanager" "wheel" "input" "uinput"];
  };
}
