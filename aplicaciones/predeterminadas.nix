{
  config,
  equipo,
  inputs,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  navegadorPredeterminado = equipo.navegadorPredeterminado or null;
  zen = inputs.zen-browser.packages.${pkgs.stdenv.hostPlatform.system}.default;

  webDefaults =
    if navegadorPredeterminado == null
    then ""
    else if navegadorPredeterminado == "zen"
    then ''
      # Web: Zen es el navegador declarativo de este equipo. Chrome nunca se
      # selecciona automáticamente; solo puede convertirse en predeterminado
      # mediante una elección manual de la persona usuaria.
      "$xdgMime" default zen-browser.desktop \
        x-scheme-handler/http \
        x-scheme-handler/https \
        text/html \
        application/xhtml+xml
    ''
    else throw "Navegador predeterminado declarativo no permitido: ${navegadorPredeterminado}. Chrome solo puede elegirse manualmente.";

  mimeDefaults = pkgs.writeShellScript "korunix-aplicaciones-predeterminadas" ''
    export XDG_CONFIG_HOME="$HOME/.config"
    export XDG_DATA_DIRS="${zen}/share:${pkgs.nautilus}/share:${pkgs.loupe}/share:${pkgs.papers}/share:${pkgs.file-roller}/share"

    xdgMime=${pkgs.xdg-utils}/bin/xdg-mime

    ${webDefaults}

    # Carpetas: Nautilus.
    "$xdgMime" default org.gnome.Nautilus.desktop \
      inode/directory

    # Imágenes: Loupe, el visor actual de GNOME.
    "$xdgMime" default org.gnome.Loupe.desktop \
      image/avif \
      image/bmp \
      image/gif \
      image/heic \
      image/heif \
      image/jpeg \
      image/jxl \
      image/png \
      image/svg+xml \
      image/tiff \
      image/webp

    # Documentos de lectura: Papers, incluido PDF y DjVu.
    "$xdgMime" default org.gnome.Papers.desktop \
      application/pdf \
      application/x-bzpdf \
      application/x-ext-pdf \
      application/x-gzpdf \
      application/x-xzpdf \
      application/x-cb7 \
      application/x-cbr \
      application/x-cbt \
      application/x-cbz \
      application/vnd.comicbook-rar \
      application/vnd.comicbook+zip \
      image/vnd.djvu \
      image/vnd.djvu+multipage

    # Archivos comprimidos: File Roller.
    "$xdgMime" default org.gnome.FileRoller.desktop \
      application/zip \
      application/x-7z-compressed \
      application/x-rar \
      application/vnd.rar \
      application/x-tar \
      application/gzip \
      application/x-gzip \
      application/x-bzip2 \
      application/x-xz \
      application/zstd \
      application/x-compressed-tar \
      application/x-bzip-compressed-tar \
      application/x-xz-compressed-tar
  '';
in {
  # Chrome puede seguir instalado, pero no se registra como predeterminado.
  # Si se usa, los PDF se entregan al sistema para que Papers los abra.
  environment.etc."opt/chrome/policies/managed/korunix.json".text = builtins.toJSON {
    AlwaysOpenPdfExternally = true;
  };

  # Aplicamos las asociaciones a la persona actual sin sustituir sus otras
  # asociaciones MIME no relacionadas con este bloque.
  system.activationScripts.aplicacionesPredeterminadas.text = ''
    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${usuario.home}/.config

    ${pkgs.util-linux}/bin/runuser \
      -u ${usuario.name} -- \
      env HOME=${usuario.home} ${mimeDefaults}
  '';
}
