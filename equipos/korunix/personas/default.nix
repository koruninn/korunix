{equipo, ...}: {
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = "André";

    # input: Bongo Cat y lectura directa de mandos.
    # uinput: Steam Input y otras capas de compatibilidad/remapeo.
    extraGroups = ["networkmanager" "wheel" "input" "uinput"];
  };
}
