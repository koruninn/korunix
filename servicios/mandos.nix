{equipo, pkgs, ...}: {
  # Dejar los mandos y el consumo de energía en manos de HHD, sin otro servicio que compita.
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
