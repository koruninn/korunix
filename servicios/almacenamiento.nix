{config, ...}: let
  usuario = config.users.users.koru.name;
  montaje = "/run/media/${usuario}/EmuDeck";
in {
  # La tarjeta debe estar disponible también en el modo consola.
  fileSystems."${montaje}" = {
    device = "/dev/disk/by-label/EmuDeck";
    fsType = "ext4";
    options = [
      "nofail"
      "x-systemd.device-timeout=5s"
      "x-systemd.device-bound"
      "nosuid"
      "nodev"
      "exec"
      "x-gvfs-show"
    ];
  };

  # Montar la tarjeta al insertarla, tanto en el lector como en el hub.
  services.udev.extraRules = ''
    ACTION=="add|change", SUBSYSTEM=="block", ENV{ID_FS_LABEL}=="EmuDeck", ENV{ID_FS_TYPE}=="ext4", TAG+="systemd", ENV{SYSTEMD_WANTS}+="run-media-${usuario}-EmuDeck.mount"
  '';
}
