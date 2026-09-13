{pkgs, ...}: {
  # Todos los equipos del repositorio se gestionan como flakes.
  nix.settings.experimental-features = ["nix-command" "flakes"];

  environment.systemPackages = with pkgs; [
    git
    curl
    wget
    tree
    just
  ];
}
