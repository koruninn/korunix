let
  catalogo = [
    {
      id = "noctalia/umbriel-companion";
      dependencias = [];
    }
    {
      id = "prponkshe/umbriel-displays";
      dependencias = [];
    }
    {
      id = "noctalia/screen_recorder";
      dependencias = ["gpu-screen-recorder"];
    }
    {
      id = "noctalia/bongocat";
      dependencias = [];
    }
    {
      id = "noctalia/timer";
      dependencias = [];
    }
    {
      id = "noctalia/kaomoji";
      dependencias = [];
    }
    {
      id = "noctalia/notes";
      dependencias = [];
    }
    {
      id = "noctalia/world_clock";
      dependencias = [];
    }
    {
      id = "aristides/udiskie";
      dependencias = ["udiskie" "xdg-utils"];
    }
    {
      id = "andrewdems/printers";
      dependencias = ["cups" "xdg-utils"];
    }
    {
      id = "nilsonlinux/speedtest-meter";
      dependencias = ["speedtest-cli"];
    }
    {
      id = "thepunkoff/pomodoro";
      dependencias = [];
    }
    {
      id = "yuuto/calculator";
      dependencias = [];
    }
    {
      id = "liamwh/emoji-picker";
      dependencias = [];
    }
    {
      id = "notfinaldev/youtube-search";
      dependencias = ["yt-dlp" "xdg-utils"];
    }
    {
      id = "icefish/phone-connect";
      dependencias = ["kdeconnect" "sshfs"];
    }
    {
      id = "dotnetrob/cat";
      dependencias = [];
    }
  ];
in {
  # ESTA ES LA LISTA FÁCIL.
  # Cada plugin aparece una sola vez junto a lo que necesita del sistema.
  activos = map (plugin: plugin.id) catalogo;
  dependencias = builtins.concatLists (map (plugin: plugin.dependencias) catalogo);

  # Copias viejas que Korunix elimina del estado local para que no vuelvan a
  # aparecer en Ajustes como si todavía estuvieran instaladas.
  descartados = [
    "community/nix-monitor"
    "community/nix-status"
    "community/drive-health"
    "community/vm-manager"
    "community/nextboot-selector"
    "community/tailnet"
    "community/taildrop"
    "community/syncthing"
    "community/file-search"
    "community/web-search"
    "community/webapp-maker"
    "community/game-launcher"
    "community/gamer-mode"
    "community/lyrics"
    "community/lrc"
    "community/spotify-lyrics"
    "community/spotify"
    "community/yt-music"
    "community/media-lyrics"
    "community/discord-voice"
    "official/translator"
    "community/thunderbird-companion"
    "community/timezone-hub"
    "community/prismlauncher-instances"
  ];
}
