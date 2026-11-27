{pkgs, ...}: {
  programs.fish = {
    enable = true;

    interactiveShellInit = ''
      set -g fish_greeting ""

      fastfetch --config /etc/fastfetch/config.jsonc
      echo
    '';
  };

  users.defaultUserShell = pkgs.fish;
}
