{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  plasmaDynamicScheme = pkgs.runCommand "korunix-plasma-dynamic-colors" {} ''
    mkdir -p "$out/share/color-schemes"

    awk '
      /^\[General\]$/ {
        print
        print "TintFactor=0.15"
        next
      }
      /^ColorScheme=BreezeDark$/ {
        print "ColorScheme=KorunixDynamic"
        next
      }
      /^Name=Breeze Dark$/ {
        print "Name=Korunix Dynamic"
        next
      }
      /^Name\[/ {
        next
      }
      { print }
    ' ${pkgs.kdePackages.breeze}/share/color-schemes/BreezeDark.colors \
      > "$out/share/color-schemes/KorunixDynamic.colors"
  '';

  plasmaNativeColors = pkgs.writeShellApplication {
    name = "korunix-plasma-colors";
    runtimeInputs = [
      pkgs.dbus
      pkgs.kdePackages.plasma-workspace
    ];
    text = ''
      # La frontera GTK se conmuta por sesión: Plasma desconecta Noctalia y
      # ChromaLeon, pero conserva sus archivos para restaurarlos al volver.
      korunix-gtk-session plasma

      # Aplicamos primero el esquema final de Plasma. Al cargar gtkconfig a
      # continuación, KDE exporta ese mismo Breeze tintado a GTK y genera su
      # colors.css de sesión.
      plasma-apply-colorscheme KorunixDynamic >/dev/null 2>&1 || true

      dbus-send \
        --session \
        --type=method_call \
        --print-reply \
        --dest=org.kde.kded6 \
        /kded \
        org.kde.kded6.loadModule \
        string:gtkconfig \
        >/dev/null 2>&1 || true

      dbus-send \
        --session \
        --type=method_call \
        --print-reply \
        --dest=org.kde.GtkConfig \
        /GtkConfig \
        org.kde.GtkConfig.setGtkTheme \
        string:Breeze \
        >/dev/null 2>&1 || true
    '';
  };
in {
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Plasma usa únicamente su motor nativo: acento derivado del fondo y un
  # Breeze oscuro teñido con ese acento. No hay generador Material You externo.
  environment.systemPackages = [
    plasmaDynamicScheme
    plasmaNativeColors
  ];

  # Dejamos preparada la selección antes de que arranque Plasma para que su
  # servicio nativo de acento pueda reaccionar al fondo desde el inicio.
  system.activationScripts.plasmaNativeColors.text = ''
    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${usuario.home}/.config

    HOME=${usuario.home} \
    XDG_CONFIG_HOME=${usuario.home}/.config \
      ${pkgs.kdePackages.kconfig}/bin/kwriteconfig6 \
        --file kdeglobals \
        --group General \
        --key ColorScheme \
        KorunixDynamic

    HOME=${usuario.home} \
    XDG_CONFIG_HOME=${usuario.home}/.config \
      ${pkgs.kdePackages.kconfig}/bin/kwriteconfig6 \
        --file kdeglobals \
        --group General \
        --key accentColorFromWallpaper \
        true

    chown ${usuario.name}:${usuario.group} ${usuario.home}/.config/kdeglobals
  '';

  environment.etc."xdg/autostart/korunix-plasma-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=Korunix Plasma Colors
Comment=Aplica los colores dinámicos nativos de Plasma
Exec=${plasmaNativeColors}/bin/korunix-plasma-colors
OnlyShowIn=KDE;
NoDisplay=true
X-KDE-AutostartScript=true
  '';

  environment.etc."xdg/kcminputrc".text = ''
[Keyboard]
NumLock=0
  '';

  environment.etc."xdg/kde-mimeapps.list".text = ''
[Default Applications]
inode/directory=org.kde.dolphin.desktop;
  '';

  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
[Containments][2][Applets][5][Configuration][General]
launchers=applications:systemsettings.desktop,applications:org.kde.dolphin.desktop,applications:zen.desktop
  '';

  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}
