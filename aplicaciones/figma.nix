{
  config,
  pkgs,
  inputs,
  ...
}:
{
  imports = [
    inputs.figma-linux-next.nixosModules.default
  ];

  programs.figma-linux-next = {
    enable = true;
  };
}
