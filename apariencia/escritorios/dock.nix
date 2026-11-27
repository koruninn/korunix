let
  comunes = [
    "firefox"
    "org.gnome.Nautilus"
    "sh.cider.Cider"
    "steam"
    "net.lutris.Lutris"
    "anime-game-launcher"
    "honkers-railway-launcher"
    "vesktop"
    "LocalSend"
    "code"
    "com.obsproject.Studio"
    "org.kde.kdenlive"
    "figma-linux-next"
    "davinci-resolve"
    "org.darktable.darktable"
    "com.heroicgameslauncher.hgl"
    "onlyoffice-desktopeditors"
    "birdfont"
    "io.github.DraqueT.PolyGlot"
    "net.donnybeelo.Convey"
  ];
in {
  # Luciole sigue instalado; falta su icono en Hatter.
  noctalia = builtins.filter (id: id != "org.gnome.Nautilus") comunes;
  # LocalSend usa el identificador de la versión nativa.
  gnome = map (id: "${id}.desktop") comunes;
}
