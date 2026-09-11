  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users."dell" = {
    isNormalUser = true;
    description = "dell";
    extraGroups = [ "networkmanager" "wheel" ];
    packages = with pkgs; [
    #  thunderbird
    ];
  };
