{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  mimeDefaults = pkgs.writeShellScript "korunix-aplicaciones-predeterminadas" ''
    export XDG_CONFIG_HOME="$HOME/.config"
    export XDG_DATA_DIRS="${pkgs.google-chrome}/share:${pkgs.nautilus}/share:${pkgs.loupe}/share:${pkgs.papers}/share:${pkgs.file-roller}/share"

    xdgMime=${pkgs.xdg-utils}/bin/xdg-mime

    # Web: Chrome sigue siendo navegador, no visor de archivos locales.
    "$xdgMime" default google-chrome.desktop \
      x-scheme-handler/http \
      x-scheme-handler/https \
      text/html \
      application/xhtml+xml

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

    # Archivos comprimidos: File Roller en lugar de PeaZip.
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
  # Chrome no captura los PDF: los descarga/entrega al sistema para que Papers
  # sea quien los abra. Las imágenes web siguen mostrándose normalmente dentro
  # de las páginas; los archivos de imagen locales quedan asociados a Loupe.
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
