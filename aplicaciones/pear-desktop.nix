{pkgs, ...}:
let
  pearDesktop = pkgs.pear-desktop.overrideAttrs (old: {
    postPatch = (old.postPatch or "") + ''
      substituteInPlace src/plugins/synced-lyrics/index.ts \
        --replace-fail \
          "    enabled: false," \
          "    enabled: true,"

      substituteInPlace src/plugins/album-color-theme/index.ts \
        --replace-fail \
          "    enabled: false," \
          "    enabled: true,"

      substituteInPlace src/plugins/do-not-track/index.ts \
        --replace-fail \
          "    enabled: false," \
          "    enabled: true,"

      substituteInPlace src/plugins/discord/index.ts \
        --replace-fail \
          "    'enabled': false," \
          "    'enabled': true,"
    '';
  });
in {
  environment.systemPackages = [pearDesktop];
}
