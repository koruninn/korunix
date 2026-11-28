{config, lib, pkgs, ...}: let
  dependencias = with pkgs; [
    bash
    coreutils
    curl
    file
    git
    jq
    lsb-release
    newt
    procps
    python3
    rsync
    SDL2
    unzip
    util-linux
    xdg-utils
    xmlstarlet
    zenity
    config.programs.appimage.package
  ];
  emudeck = pkgs.writeShellScriptBin "emudeck" ''
    export PATH=${lib.makeBinPath dependencias}:$PATH
    ${builtins.readFile ./emudeck.sh}
  '';
  escritorio = pkgs.makeDesktopItem {
    name = "korunix-emudeck";
    desktopName = "EmuDeck";
    exec = "${emudeck}/bin/emudeck";
    icon = "applications-games";
    categories = ["Game" "Emulator"];
    terminal = true;
  };
in {
  environment.systemPackages = dependencias ++ [emudeck escritorio];
}
