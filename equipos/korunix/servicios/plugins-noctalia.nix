{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  pythonNoctalia = pkgs.python3.withPackages (pythonPackages: [
    pythonPackages.syncedlyrics
  ]);

  lrcTty = pkgs.stdenv.mkDerivation {
    pname = "lrc_tty";
    version = "0.8";

    src = pkgs.fetchurl {
      name = "lrc_tty-0.8.tar.gz";
      url = "https://github.com/larsgrah/lrc_tty/archive/refs/tags/v0.8.tar.gz";
      hash = "sha256-69QUd3B0skp7yr5Xx8Hjw6cmzvBfr+BITOCXfi7wUsQ=";
    };

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.zig_0_16
    ];
    buildInputs = [pkgs.dbus];

    buildPhase = ''
      runHook preBuild
      export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-global-cache"
      export ZIG_LOCAL_CACHE_DIR="$TMPDIR/zig-local-cache"
      zig build -Doptimize=ReleaseFast
      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      install -Dm755 zig-out/bin/lrc_tty "$out/bin/lrc_tty"
      runHook postInstall
    '';
  };

  mpvNoctalia = pkgs.mpv.override {
    scripts = [pkgs.mpvScripts.mpris];
  };

  spotifyLyricsDaemon = pkgs.writeShellApplication {
    name = "noctalia-spotify-lyrics-daemon";
    runtimeInputs = [
      pkgs.coreutils
      pythonNoctalia
    ];
    text = ''
      daemon="$HOME/.local/state/noctalia/plugins/materialized/community/spotify-lyrics/spotify_lyrics_daemon.py"
      while [ ! -f "$daemon" ]; do
        sleep 5
      done
      exec python3 "$daemon"
    '';
  };
in {
  # Herramientas externas que usan los plugins seleccionados de Noctalia.
  environment.systemPackages = [
    pkgs.coreutils
    pkgs.cups
    pkgs.discord
    pkgs.efibootmgr
    pkgs.evtest
    pkgs.ffmpeg
    pkgs.file
    pkgs.findutils
    pkgs.fzf
    pkgs.gawk
    pkgs.gcc
    pkgs.glib
    pkgs.gnugrep
    pkgs.gnused
    pkgs.gnutar
    pkgs.gzip
    pkgs.gpu-screen-recorder
    pkgs.gtk3
    pkgs.jq
    pkgs.kdePackages.kdeconnect-kde
    pkgs.libnotify
    pkgs.libvirt
    pkgs.netcat-openbsd
    pkgs.openssh
    pkgs.playerctl
    pkgs.polkit
    pkgs.power-profiles-daemon
    pkgs.procps
    pkgs.smartmontools
    pkgs.speedtest-cli
    pkgs.spotify-player
    pkgs.sshfs
    pkgs.sudo
    pkgs.syncthing
    pkgs.udiskie
    pkgs.util-linux
    pkgs.virt-viewer
    pkgs.xdg-user-dirs
    pkgs.xdg-utils
    pkgs.yt-dlp
    lrcTty
    mpvNoctalia
    pythonNoctalia
  ];

  security.polkit.enable = true;
  virtualisation.libvirtd.enable = true;

  # Bongo Cat necesita leer dispositivos de entrada y VM Manager acceder a libvirt.
  users.users.${equipo.persona}.extraGroups = [
    "input"
    "libvirtd"
  ];

  services.tailscale.extraSetFlags = ["--operator=${equipo.persona}"];

  services.syncthing = {
    enable = true;
    user = usuario.name;
    group = usuario.group;
    dataDir = "${usuario.home}/Sync";
    configDir = "${usuario.home}/.config/syncthing";
    openDefaultPorts = true;
  };

  # Spotify Lyrics necesita su pequeño proceso auxiliar además del plugin.
  systemd.user.services.noctalia-spotify-lyrics = {
    description = "Letras sincronizadas para Noctalia";
    wantedBy = ["default.target"];
    after = ["graphical-session.target"];
    serviceConfig = {
      ExecStart = "${spotifyLyricsDaemon}/bin/noctalia-spotify-lyrics-daemon";
      Restart = "on-failure";
      RestartSec = 5;
    };
  };

  # La lista de plugins pertenece a Korunix; sus ajustes individuales siguen siendo editables.
  system.activationScripts.noctaliaPlugins = {
    deps = ["noctaliaConfig"];
    text = ''
      settings_file=${usuario.home}/.local/state/noctalia/settings.toml
      if [ -f "$settings_file" ]; then
        tmp_file="$settings_file.korunix-plugins-tmp"
        skip_plugins=false
        while IFS= read -r line || [ -n "$line" ]; do
          case "$line" in
            "[plugins]")
              skip_plugins=true
              continue
              ;;
            "["*)
              skip_plugins=false
              ;;
          esac
          if [ "$skip_plugins" = false ]; then
            printf '%s\n' "$line"
          fi
        done < "$settings_file" > "$tmp_file"
        install -m 0644 -o ${usuario.name} -g ${usuario.group} "$tmp_file" "$settings_file"
        rm -f "$tmp_file"
      fi
    '';
  };
}
