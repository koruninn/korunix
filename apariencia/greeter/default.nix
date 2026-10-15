{ pkgs, ... }:
{
  services.displayManager.gdm.enable = true;

  nixpkgs.overlays = [
    (final: prev: {
      gnome-shell = prev.gnome-shell.overrideAttrs (old: {
        patches = (old.patches or []) ++ [
          (final.replaceVars ./gdm-background.patch {
            backgroundPath =
              "${final.gnome-backgrounds}/share/backgrounds/gnome/adwaita-d.jxl";
          })
        ];
      });
    })
  ];
}
