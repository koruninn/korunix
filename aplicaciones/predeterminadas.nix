{
  equipo,
  lib,
  ...
}: let
  navegadorPredeterminado = equipo.navegadorPredeterminado or null;

  webDefaults =
    if navegadorPredeterminado == null
    then {}
    else if navegadorPredeterminado == "firefox"
    then {
      "x-scheme-handler/http" = "firefox.desktop";
      "x-scheme-handler/https" = "firefox.desktop";
      "text/html" = "firefox.desktop";
      "application/xhtml+xml" = "firefox.desktop";
    }
    else throw "Navegador predeterminado declarativo no permitido: ${navegadorPredeterminado}. Chrome solo puede elegirse manualmente.";
in {
  # Los cambios personales en mimeapps.list tienen prioridad.
  xdg.mime.defaultApplications =
    webDefaults
    // {
      "inode/directory" = "org.gnome.Nautilus.desktop";

      "image/avif" = "org.gnome.Loupe.desktop";
      "image/bmp" = "org.gnome.Loupe.desktop";
      "image/gif" = "org.gnome.Loupe.desktop";
      "image/heic" = "org.gnome.Loupe.desktop";
      "image/heif" = "org.gnome.Loupe.desktop";
      "image/jpeg" = "org.gnome.Loupe.desktop";
      "image/jxl" = "org.gnome.Loupe.desktop";
      "image/png" = "org.gnome.Loupe.desktop";
      "image/svg+xml" = "org.gnome.Loupe.desktop";
      "image/tiff" = "org.gnome.Loupe.desktop";
      "image/webp" = "org.gnome.Loupe.desktop";

      "application/pdf" = "org.gnome.Papers.desktop";
      "application/x-bzpdf" = "org.gnome.Papers.desktop";
      "application/x-ext-pdf" = "org.gnome.Papers.desktop";
      "application/x-gzpdf" = "org.gnome.Papers.desktop";
      "application/x-xzpdf" = "org.gnome.Papers.desktop";
      "application/x-cb7" = "org.gnome.Papers.desktop";
      "application/x-cbr" = "org.gnome.Papers.desktop";
      "application/x-cbt" = "org.gnome.Papers.desktop";
      "application/x-cbz" = "org.gnome.Papers.desktop";
      "application/vnd.comicbook-rar" = "org.gnome.Papers.desktop";
      "application/vnd.comicbook+zip" = "org.gnome.Papers.desktop";
      "image/vnd.djvu" = "org.gnome.Papers.desktop";
      "image/vnd.djvu+multipage" = "org.gnome.Papers.desktop";

      "application/zip" = "org.gnome.FileRoller.desktop";
      "application/x-7z-compressed" = "org.gnome.FileRoller.desktop";
      "application/x-rar" = "org.gnome.FileRoller.desktop";
      "application/vnd.rar" = "org.gnome.FileRoller.desktop";
      "application/x-tar" = "org.gnome.FileRoller.desktop";
      "application/gzip" = "org.gnome.FileRoller.desktop";
      "application/x-gzip" = "org.gnome.FileRoller.desktop";
      "application/x-bzip2" = "org.gnome.FileRoller.desktop";
      "application/x-xz" = "org.gnome.FileRoller.desktop";
      "application/zstd" = "org.gnome.FileRoller.desktop";
      "application/x-compressed-tar" = "org.gnome.FileRoller.desktop";
      "application/x-bzip-compressed-tar" = "org.gnome.FileRoller.desktop";
      "application/x-xz-compressed-tar" = "org.gnome.FileRoller.desktop";
    };

  # Dejar que Papers abra los PDF descargados desde Chrome.
  environment.etc."opt/chrome/policies/managed/korunix.json".text = builtins.toJSON {
    AlwaysOpenPdfExternally = true;
  };
}
