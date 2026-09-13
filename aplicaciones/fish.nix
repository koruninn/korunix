{pkgs, ...}: {
  # Habilita Fish y lo deja como shell de inicio por defecto para los usuarios
  # normales que no definan otro shell explícitamente.
  programs.fish.enable = true;
  users.defaultUserShell = pkgs.fish;
}
