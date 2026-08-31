{
  lib,
  pkgs,
}: let
  productDefaults = import ./predeterminados.nix;
  productPolicy = productDefaults.roles;

  catalog = {
    alacritty = {
      package = pkgs.alacritty;
      desktop = "Alacritty.desktop";
      executable = "alacritty";
    };

    fish = {
      package = pkgs.fish;
      desktop = null;
      executable = "fish";
    };

    thunderbird = {
      package = pkgs.thunderbird;
      desktop = "thunderbird.desktop";
      executable = "thunderbird";
    };

    firefox = {
      package = pkgs.firefox;
      desktop = "firefox.desktop";
      executable = "firefox";
    };

    "google-chrome" = {
      package = pkgs.google-chrome;
      desktop = "google-chrome.desktop";
      executable = "google-chrome-stable";
    };

    nautilus = {
      package = pkgs.nautilus;
      desktop = "org.gnome.Nautilus.desktop";
      executable = "nautilus";
    };

    loupe = {
      package = pkgs.loupe;
      desktop = "org.gnome.Loupe.desktop";
      executable = "loupe";
    };

    papers = {
      package = pkgs.papers;
      desktop = "org.gnome.Papers.desktop";
      executable = "papers";
    };

    "gnome-text-editor" = {
      package = pkgs.gnome-text-editor;
      desktop = "org.gnome.TextEditor.desktop";
      executable = "gnome-text-editor";
    };

    nemo = {
      package = pkgs.nemo-with-extensions;
      desktop = "nemo.desktop";
      executable = "nemo";
    };

    xviewer = {
      package = pkgs.xviewer;
      desktop = "xviewer.desktop";
      executable = "xviewer";
    };

    xreader = {
      package = pkgs.xreader;
      desktop = "xreader.desktop";
      executable = "xreader";
    };

    xed = {
      package = pkgs.xed-editor;
      desktop = "xed.desktop";
      executable = "xed";
    };

    dolphin = {
      package = pkgs.kdePackages.dolphin;
      desktop = "org.kde.dolphin.desktop";
      executable = "dolphin";
    };

    gwenview = {
      package = pkgs.kdePackages.gwenview;
      desktop = "org.kde.gwenview.desktop";
      executable = "gwenview";
    };

    okular = {
      package = pkgs.kdePackages.okular;
      desktop = "org.kde.okular.desktop";
      executable = "okular";
    };

    kwrite = {
      package = pkgs.kdePackages.kate;
      desktop = "org.kde.kwrite.desktop";
      executable = "kwrite";
    };

    kate = {
      package = pkgs.kdePackages.kate;
      desktop = "org.kde.kate.desktop";
      executable = "kate";
    };
  };

  appFor = id:
    catalog.${id}
    or (throw "Korunix no tiene traducción técnica para el rol de aplicación ${id}.");

  roleFor = desktop: role: let
    desktopRoles = productPolicy.byDesktop.${desktop} or {};
    id =
      desktopRoles.${role}
      or productPolicy.common.${role}
      or null;
  in
    if id == null
    then null
    else appFor id;

  fixedRoleNames = [
    "terminal"
    "shell"
    "mail"
    "fileManager"
    "imageViewer"
    "pdfViewer"
    "textEditor"
  ];

  fixedIdsForDesktop = desktop:
    lib.filter
    (id: id != null)
    (map (
        role: let
          app = roleFor desktop role;
        in
          if app == null
          then null
          else (productPolicy.byDesktop.${desktop}.${role}
              or productPolicy.common.${role})
      )
      fixedRoleNames);

  mimeTypes = {
    browser = [
      "x-scheme-handler/http"
      "x-scheme-handler/https"
      "x-scheme-handler/chrome"
      "text/html"
      "application/xhtml+xml"
      "application/x-extension-htm"
      "application/x-extension-html"
      "application/x-extension-shtml"
      "application/x-extension-xhtml"
      "application/x-extension-xht"
    ];

    fileManager = [
      "inode/directory"
    ];

    imageViewer = [
      "image/avif"
      "image/bmp"
      "image/gif"
      "image/heic"
      "image/jpeg"
      "image/jpg"
      "image/jxl"
      "image/pjpeg"
      "image/png"
      "image/svg+xml"
      "image/svg+xml-compressed"
      "image/tiff"
      "image/vnd.wap.wbmp"
      "image/webp"
      "image/x-bmp"
      "image/x-gray"
      "image/x-icb"
      "image/x-icns"
      "image/x-ico"
      "image/x-pcx"
      "image/x-png"
      "image/x-portable-anymap"
      "image/x-portable-bitmap"
      "image/x-portable-graymap"
      "image/x-portable-pixmap"
      "image/x-xbitmap"
      "image/x-xpixmap"
    ];

    pdfViewer = [
      "application/pdf"
    ];

    textEditor = [
      "text/plain"
    ];

    mail = [
      "x-scheme-handler/mailto"
      "x-scheme-handler/mid"
      "message/rfc822"
    ];
  };

  mimeRoleNames = [
    "browser"
    "fileManager"
    "imageViewer"
    "pdfViewer"
    "textEditor"
    "mail"
  ];

  desktopMimeFileNames = {
    niri = "niri-mimeapps.list";
    hyprland = "hyprland-mimeapps.list";
    cinnamon = "x-cinnamon-mimeapps.list";
    plasma = "kde-mimeapps.list";
  };

  mimeDefaultsFor = {
    desktop,
    browser ? null,
    plasmaTextEditor ? null,
  }: let
    selected = {
      browser =
        if browser == null
        then null
        else appFor browser;

      fileManager = roleFor desktop "fileManager";
      imageViewer = roleFor desktop "imageViewer";
      pdfViewer = roleFor desktop "pdfViewer";

      textEditor =
        if desktop == "plasma"
        then
          if plasmaTextEditor == null
          then null
          else appFor plasmaTextEditor
        else roleFor desktop "textEditor";

      mail = roleFor desktop "mail";
    };

    addRole = defaults: role: let
      app = selected.${role};
      types = mimeTypes.${role};
    in
      if app == null
      then defaults
      else
        defaults
        // lib.genAttrs types (_: app.desktop);
  in
    lib.foldl' addRole {} mimeRoleNames;
in {
  inherit
    catalog
    desktopMimeFileNames
    mimeDefaultsFor
    productPolicy
    ;

  browserChoices = productPolicy.choices.browser;
  plasmaTextEditorChoices = productPolicy.choices.plasmaTextEditor;

  packagesFor = desktops:
    map
    (id: (appFor id).package)
    (lib.unique (lib.concatMap fixedIdsForDesktop desktops));

  packageFor = id: (appFor id).package;
}
