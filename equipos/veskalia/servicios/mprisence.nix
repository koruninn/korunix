{
  pkgs,
  ...
}: {
  systemd.user.services.mprisence = {
    description = "Discord Rich Presence para reproductores MPRIS";
    wantedBy = ["default.target"];

    serviceConfig = {
      ExecStart = "${pkgs.mprisence}/bin/mprisence";
      Restart = "on-failure";
      RestartSec = 5;
    };
  };
}
