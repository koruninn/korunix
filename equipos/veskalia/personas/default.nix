{equipo, ...}: {
  users.users.${equipo.persona} = {
    isNormalUser = true;
    description = "André";

    # input deja que aplicaciones como Bongo Cat puedan leer directamente algunos dispositivos,
    # incluidos mandos que necesitan acceso a los eventos de entrada del equipo.
    # uinput deja que Steam Input y otras herramientas puedan crear eventos de entrada para remapear mandos.
    extraGroups = ["networkmanager" "wheel" "input" "uinput"];
  };
}
