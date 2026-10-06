{pkgs, ...}: {
  programs.nix-ld = {
    enable = true;

    libraries =
      (pkgs.steam-run.args.multiPkgs pkgs)
      ++ (with pkgs; [
        xorg.libX11
        xorg.libXext
        xorg.libXrandr
        xorg.libXcursor
        xorg.libXi
        xorg.libXinerama
        xorg.libXfixes
        xorg.libXrender
        xorg.libXdamage
        xorg.libXScrnSaver
        xorg.libXxf86vm
        xorg.libxcb

        libGL
        libGLU
        stdenv.cc.cc
      ]);
  };

  programs.appimage = {
    enable = true;
    binfmt = true;
  };
}
