{
  config,
  pkgs,
  ...
}: let
  materialYou = pkgs.python3Packages."kde-material-you-colors";
  noctaliaPackage = config.programs.noctalia.package;

  plasmaThemeIsolation = pkgs.writeShellApplication {
    name = "korunix-plasma-theme";
    runtimeInputs = with pkgs; [
      bash
      blender
      coreutils
      findutils
      gawk
      gnugrep
      gnused
      jq
      libreoffice
    ];
    text = ''
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      data_home="''${XDG_DATA_HOME:-$HOME/.local/share}"
      noctalia_templates="${noctaliaPackage}/share/noctalia/assets/templates"

      remove_file() {
        local file="$1"
        if [ -e "$file" ] || [ -L "$file" ]; then
          rm -f -- "$file"
        fi
      }

      run_undo() {
        local script="$1"
        if [ -f "$script" ]; then
          bash "$script" >/dev/null 2>&1 || true
        fi
      }

      strip_zen_imports() {
        local root prefs profile chrome_file content_file tmp

        for root in "$config_home/zen" "$HOME/.zen"; do
          [ -d "$root" ] || continue
          while IFS= read -r -d ''' prefs; do
            profile=$(dirname "$prefs")
            chrome_file="$profile/chrome/userChrome.css"
            content_file="$profile/chrome/userContent.css"

            if [ -f "$chrome_file" ]; then
              tmp=$(mktemp)
              awk '!/noctalia\/zen-browser\/zen-userChrome\.css/' "$chrome_file" > "$tmp"
              if ! cmp -s "$chrome_file" "$tmp"; then
                cat "$tmp" > "$chrome_file"
              fi
              rm -f "$tmp"
            fi

            if [ -f "$content_file" ]; then
              tmp=$(mktemp)
              awk '!/noctalia\/zen-browser\/zen-userContent\.css/' "$content_file" > "$tmp"
              if ! cmp -s "$content_file" "$tmp"; then
                cat "$tmp" > "$content_file"
              fi
              rm -f "$tmp"
            fi
          done < <(find "$root" -mindepth 2 -maxdepth 2 -type f -name prefs.js -print0 2>/dev/null || true)
        done
      }

      disable_obsidian_snippets() {
        local obsidian_dir appearance tmp

        while IFS= read -r -d ''' obsidian_dir; do
          appearance="$obsidian_dir/appearance.json"
          [ -f "$appearance" ] || continue
          tmp=$(mktemp)
          if jq 'if (.enabledCssSnippets? | type) == "array" then .enabledCssSnippets |= map(select(. != "noctalia")) else . end' "$appearance" > "$tmp"; then
            if ! cmp -s "$appearance" "$tmp"; then
              cat "$tmp" > "$appearance"
            fi
          fi
          rm -f "$tmp"
        done < <(find "$HOME" -maxdepth 4 -type d -name .obsidian -print0 2>/dev/null || true)
      }

      reset_blender_theme() {
        if command -v blender >/dev/null 2>&1; then
          blender --background --python-expr 'import bpy; bpy.ops.preferences.reset_default_theme(); bpy.ops.wm.save_userpref()' >/dev/null 2>&1 || true
        fi
      }

      disable_libreoffice_theme() {
        if command -v unopkg >/dev/null 2>&1; then
          unopkg remove dev.noctalia.libreoffice.theme >/dev/null 2>&1 || true
        fi

        if command -v flatpak >/dev/null 2>&1 && flatpak info org.libreoffice.LibreOffice >/dev/null 2>&1; then
          flatpak run --command=/app/libreoffice/program/unopkg org.libreoffice.LibreOffice \
            remove dev.noctalia.libreoffice.theme >/dev/null 2>&1 || true
        fi
      }

      # Plantillas builtin: usa los undo oficiales de Noctalia para conservar
      # cualquier otra configuración de Alacritty y GTK.
      run_undo "$noctalia_templates/alacritty/undo.sh"
      run_undo "$noctalia_templates/gtk/undo-gtk3.sh"
      run_undo "$noctalia_templates/gtk/undo-gtk4.sh"
      run_undo "$noctalia_templates/qt/undo.sh"

      # Plantillas comunitarias que solo escriben archivos de tema. Al volver a
      # Niri/Umbriel, Noctalia los genera de nuevo con la paleta vigente.
      remove_file "$config_home/darktable/themes/noctalia.css"
      remove_file "$HOME/.var/app/org.darktable.Darktable/config/darktable/themes/noctalia.css"

      for cord_dir in \
        "$config_home/vesktop/themes" \
        "$config_home/webcord/themes" \
        "$config_home/legcord/themes" \
        "$config_home/equibop/themes" \
        "$config_home/Equicord/themes" \
        "$config_home/lightcord/themes" \
        "$config_home/dorion/themes" \
        "$config_home/Vencord/themes" \
        "$config_home/BetterDiscord/themes" \
        "$HOME/.var/app/dev.vencord.Vesktop/config/vesktop/themes" \
        "$HOME/.var/app/com.discordapp.Discord/config/Vencord/themes"
      do
        remove_file "$cord_dir/noctalia.theme.css"
        remove_file "$cord_dir/noctalia-material.theme.css"
        remove_file "$cord_dir/discord-system24.css"
      done

      for file in "$config_home"/GIMP/*/gimp.css "$HOME"/.var/app/org.gimp.GIMP/config/GIMP/*/gimp.css; do
        [ -e "$file" ] || continue
        remove_file "$file"
      done

      remove_file "$config_home/heroic/themes/matugen.css"
      remove_file "$HOME/.var/app/com.heroicgameslauncher.hgl/config/heroic/themes/matugen.css"
      remove_file "$config_home/inkscape/ui/user.css"
      remove_file "$HOME/.var/app/org.inkscape.Inkscape/config/inkscape/ui/user.css"
      remove_file "$config_home/obs-studio/themes/matugen.obt"
      remove_file "$HOME/.var/app/com.obsproject.Studio/config/obs-studio/themes/matugen.obt"
      remove_file "$data_home/PrismLauncher/themes/Matugen/theme.json"
      remove_file "$HOME/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/themes/Matugen/theme.json"
      remove_file "$HOME/.steam/steam/steamui/skins/Material-Theme/css/main/colors/matugen.css"

      for file in \
        "$HOME"/.vscode/extensions/noctalia.noctaliatheme-*/themes/NoctaliaTheme-color-theme.json \
        "$HOME"/.vscode-oss/extensions/noctalia.noctaliatheme-*/themes/NoctaliaTheme-color-theme.json \
        "$HOME"/.antigravity-ide/extensions/noctalia.noctaliatheme-*/themes/NoctaliaTheme-color-theme.json
      do
        [ -e "$file" ] || continue
        remove_file "$file"
      done

      # Estas plantillas modifican estado interno además de escribir un archivo.
      # Revertimos únicamente la parte de tema; el resto de preferencias queda intacto.
      disable_obsidian_snippets
      strip_zen_imports
      reset_blender_theme
      disable_libreoffice_theme
    '';
  };
in {
  # Plasma 6 únicamente como sesión Wayland.
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Plasma conserva su propio motor de colores y retira al iniciar únicamente
  # los artefactos que Noctalia deja persistentes en el HOME compartido.
  environment.systemPackages = [
    materialYou
    plasmaThemeIsolation
  ];

  environment.etc."xdg/autostart/korunix-plasma-theme.desktop".text = ''
[Desktop Entry]
Type=Application
Name=Korunix Plasma Theme Isolation
Comment=Separa las plantillas de Noctalia de la sesión Plasma
Exec=${plasmaThemeIsolation}/bin/korunix-plasma-theme
OnlyShowIn=KDE;
NoDisplay=true
X-KDE-AutostartScript=true
  '';

  environment.etc."xdg/autostart/kde-material-you-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=KDE Material You Colors
Comment=Colores dinámicos de Plasma a partir del fondo
Exec=${materialYou}/bin/kde-material-you-colors
Icon=color-management
OnlyShowIn=KDE;
X-KDE-AutostartScript=true
  '';

  # Num Lock encendido al iniciar Plasma.
  environment.etc."xdg/kcminputrc".text = ''
[Keyboard]
NumLock=0
  '';

  # Plasma no debe copiar su apariencia hacia GTK.
  environment.etc."xdg/kded5rc".text = ''
[Module-gtkconfig]
autoload=false
  '';

  # Plasma usa Dolphin como gestor de archivos sin cambiar Umbriel.
  environment.etc."xdg/kde-mimeapps.list".text = ''
[Default Applications]
inode/directory=org.kde.dolphin.desktop;
  '';

  # Orden declarativo de los lanzadores fijados del panel de Plasma.
  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
[Containments][2][Applets][5][Configuration][General]
launchers=applications:systemsettings.desktop,applications:org.kde.dolphin.desktop,applications:zen.desktop
  '';

  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}
