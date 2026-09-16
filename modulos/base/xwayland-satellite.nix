{lib, pkgs, ...}: {
  # Excepción deliberada: Korunix mantiene 0.8.2 mientras versiones posteriores
  # den problemas. El resto de componentes sigue upstream mediante flake.lock.
  nixpkgs.overlays = [
    (final: prev: {
      xwayland-satellite = final.rustPlatform.buildRustPackage (finalAttrs: {
        pname = "xwayland-satellite";
        version = "0.8.2";

        src = final.fetchFromGitHub {
          owner = "Supreeeme";
          repo = "xwayland-satellite";
          tag = "v${finalAttrs.version}";
          hash = "sha256-Mb7jpqnrcYCfNSItIkkHpuR3YxWFxPuIBfcwNKlRBkk=";
        };

        postPatch = ''
          substituteInPlace resources/xwayland-satellite.service \
            --replace-fail '/usr/local/bin' "$out/bin"
        '';

        cargoHash = "sha256-Saa3SRsQuY6u6pfBGezaEExOt/ReblnrG7pAXjA6Dk8=";

        nativeBuildInputs = [
          final.installShellFiles
          final.makeBinaryWrapper
          final.pkg-config
          final.rustPlatform.bindgenHook
        ];

        buildInputs = [
          final.libxcb
          final.libxcb-cursor
        ];

        buildNoDefaultFeatures = true;
        buildFeatures = ["systemd"];
        outputs = ["out" "man"];
        doCheck = false;

        postInstall = ''
          installManPage --name xwayland-satellite.1 xwayland-satellite.man
          install -Dm0644 resources/xwayland-satellite.service -t $out/lib/systemd/user
        '';

        postFixup = ''
          wrapProgram $out/bin/xwayland-satellite \
            --prefix PATH : "${lib.makeBinPath [final.xwayland]}"
        '';

        meta = {
          description = "Xwayland outside your Wayland compositor";
          homepage = "https://github.com/Supreeeme/xwayland-satellite";
          license = lib.licenses.mpl20;
          mainProgram = "xwayland-satellite";
          platforms = lib.platforms.linux;
        };
      });
    })
  ];
}
