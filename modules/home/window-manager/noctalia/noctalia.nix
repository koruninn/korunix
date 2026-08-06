{
  config,
  pkgs,
  inputs,
  ...
}: {
  imports = [
    inputs.noctalia.homeModules.default
    ./settings/bar.nix
    ./settings/dock.nix
    ./settings/lockscreen.nix
    ./settings/osd.nix
    ./settings/services.nix
    ./settings/shell.nix
    ./settings/theme.nix
    ./settings/wallpaper.nix
  ];

  programs.noctalia.enable = true;
}
