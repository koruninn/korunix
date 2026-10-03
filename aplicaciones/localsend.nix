{
  config,
  pkgs,
  inputs,
  ...
}: {
  programs.localsend = {
    enable = true;
    openFirewall = true;
  };
}
