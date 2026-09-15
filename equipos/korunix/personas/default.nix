{equipo, ...}: {
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = "André";

    # input se queda: Bongo Cat lee dispositivos de entrada y los mandos de
    # Xbox también forman parte del uso normal de Korunix.
    extraGroups = ["networkmanager" "wheel" "input"];
  };
}
