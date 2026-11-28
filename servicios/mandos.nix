{equipo, pkgs, ...}: {
  # HHD controla los mandos y el consumo de energía.
  services.inputplumber.enable = false;
  services.power-profiles-daemon.enable = pkgs.lib.mkForce false;
  services.handheld-daemon = {
    enable = true;
    user = "koru";
    ui.enable = true;
    adjustor = {
      enable = true;
      loadAcpiCallModule = true;
    };
  };

  # Dejar la suspensión en manos del sistema, sin cambiar los demás ajustes de HHD.
  systemd.services.handheld-daemon.preStart = ''
    ${pkgs.python3.withPackages (p: [p.pyyaml])}/bin/python3 ${./hhd-encendido.py}
  '';

  hardware.graphics = {
    enable = true;
    enable32Bit = true;
  };

  environment.systemPackages = with pkgs; [
    mangohud
    vulkan-tools
    evtest
  ];
}
