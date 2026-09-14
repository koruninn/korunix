{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  noctaliaPackage = config.programs.noctalia.package;

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
      pkgs.bash
      pkgs.glib
      pkgs.kdePackages.plasma-workspace
    ];
    text = ''
      noctalia_templates="${noctaliaPackage}/share/noctalia/assets/templates"

      for undo in \
        "$noctalia_templates/gtk/undo-gtk3.sh" \
        "$noctalia_templates/gtk/undo-gtk4.sh" \
        "$noctalia_templates/qt/undo.sh"
      do
        if [ -f "$undo" ]; then
          bash "$undo" >/dev/null 2>&1 || true
        fi
      done

      if command -v gsettings >/dev/null 2>&1; then
        gsettings set org.gnome.desktop.interface gtk-theme 'Breeze' >/dev/null 2>&1 || true
      fi

      plasma-apply-colorscheme KorunixDynamic >/dev/null 2>&1 || true
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

  # kde-gtk-config queda disponible para que las aplicaciones GTK de Plasma
  # sigan el esquema Breeze y su acento dinámico.
  environment.etc."xdg/kded5rc".text = ''
[Module-gtkconfig]
autoload=true
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
