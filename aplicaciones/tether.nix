{config, inputs, pkgs, ...}: {
  imports = [inputs.tether.nixosModules.default ./tether-camara.nix];

  programs.tether = {
    enable = true;
    package = inputs.tether.packages.${pkgs.stdenv.hostPlatform.system}.default;
    wifi = {
      enable = true;
      openFirewall = true;
    };
    bluetooth = {
      enable = true;
      adapters = ["hci0"];
    };
  };

  systemd.user.services.tetherd = {
    description = "Conectar el iPhone con Linux";
    wantedBy = ["default.target"];
    after = ["dbus.socket"];
    unitConfig.ConditionUser = config.users.users.koru.name;
    serviceConfig = {
      ExecStart = "${config.programs.tether.package}/bin/tetherd";
      Restart = "on-failure";
      RestartSec = 3;
    };
  };
}
