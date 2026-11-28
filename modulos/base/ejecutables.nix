{config, lib, pkgs, ...}: let
  # Compatibilidad con programas externos.
  extras = p: with p; [
    libglvnd
    libxkbcommon
    libdecor
    alsa-lib
    pipewire
    fuse
    fuse3
    glib-networking
    libsecret
    e2fsprogs
    libgpg-error
    stdenv.cc.cc
  ];
  appimage = config.programs.appimage.package;
  ejecutar = pkgs.writeShellScript "korunix-appimage-directo" ''
    set -eu
    if [ "$#" -eq 0 ]; then exit 1; fi
    imagen="$(${pkgs.coreutils}/bin/realpath -- "$1")"
    shift
    if [ ! -f "$imagen" ] || [ ! -x "$imagen" ]; then
      echo "Korunix: el AppImage debe tener permiso de ejecución: $imagen" >&2
      exit 1
    fi
    exec ${lib.getExe appimage} "$imagen" "$@"
  '';
  escritorio = pkgs.makeDesktopItem {
    name = "korunix-appimage";
    desktopName = "Ejecutar AppImage";
    exec = "${ejecutar} %f";
    icon = "application-x-executable";
    terminal = false;
    noDisplay = true;
    mimeTypes = ["application/vnd.appimage" "application/x-iso9660-appimage"];
    categories = ["Utility"];
  };
in {
  programs.nix-ld = {
    enable = true;
    libraries = lib.unique (
      (pkgs.steam-run.args.multiPkgs pkgs)
      ++ (pkgs.appimageTools.defaultFhsEnvArgs.targetPkgs pkgs)
      ++ (pkgs.appimageTools.defaultFhsEnvArgs.multiPkgs pkgs)
      ++ extras pkgs
    );
  };

  programs.appimage = {
    enable = true;
    package = pkgs.appimage-run.override {extraPkgs = extras;};
    binfmt = false;
  };

  boot.binfmt.registrations = lib.genAttrs [
    "AppImage" "appimage" "APPIMAGE"
  ] (extension: {
    recognitionType = "extension";
    magicOrExtension = extension;
    interpreter = ejecutar;
    wrapInterpreterInShell = false;
  });

  environment.systemPackages = [escritorio pkgs.steam-run];
  xdg.mime.defaultApplications = {
    "application/vnd.appimage" = lib.mkDefault "korunix-appimage.desktop";
    "application/x-iso9660-appimage" = lib.mkDefault "korunix-appimage.desktop";
  };
}
