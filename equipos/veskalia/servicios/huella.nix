{
  inputs,
  lib,
  ...
}: {
  nixpkgs.overlays = [
    (final: prev: {
      libfprint = prev.libfprint.overrideAttrs (old: {
        version = "1.94.9-sdcp-ally";
        outputs = ["out"];
        src = inputs.libfprint-egismoc-sdcp;

        patches = [
          "${inputs.ally-fingerprint-patches}/patches/0001-egismoc-add-0588-and-fix-openssl-link.patch"
          "${inputs.ally-fingerprint-patches}/patches/0002-egismoc-fix-daemon-abort-on-cancelled-verify.patch"
        ];

        mesonFlags =
          (map
            (flag:
              if lib.hasPrefix "-Ddrivers=" flag
              then "-Ddrivers=egismoc"
              else flag)
            (old.mesonFlags or []))
          ++ [
            "-Dintrospection=false"
            "-Ddoc=false"
            "-Dgtk-examples=false"
          ];
      });
    })
  ];

  # Servicio de huellas para KDE Plasma y futuras integraciones PAM.
  services.fprintd.enable = true;

  # Evita que el ahorro de energía USB suspenda este sensor.
  services.udev.extraRules = ''
    ACTION=="add", SUBSYSTEM=="usb", ATTR{idVendor}=="1c7a", ATTR{idProduct}=="0588", TEST=="power/control", ATTR{power/control}="on"
  '';
}
