{
  pkgs,
  inputs,
  ...
}: let
  figma = inputs.figma-linux-next.packages.${pkgs.stdenv.hostPlatform.system}.default;
in {
  environment.systemPackages = [
    (figma.overrideAttrs (old: {
      nativeBuildInputs =
        (old.nativeBuildInputs or [])
        ++ [
          pkgs.makeWrapper
        ];

      postFixup =
        (old.postFixup or "")
        + ''
          wrapProgram $out/bin/figma-linux-next \
            --add-flags "--ozone-platform=x11 --ignore-gpu-blocklist"
        '';
    }))
  ];

  xdg.mime.defaultApplications."x-scheme-handler/figma" = "figma-linux-next.desktop";
}
