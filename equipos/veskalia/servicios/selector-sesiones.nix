{ lib, pkgs, ... }:
let
  gtkPython = pkgs.python3.withPackages (pythonPackages: [
    pythonPackages.pygobject3
  ]);

  selector = pkgs.writeShellApplication {
    name = "korunix-session-selector";
    runtimeInputs = [ gtkPython pkgs.steamos-manager ];
    text = ''
      export GI_TYPELIB_PATH="${lib.makeSearchPath "lib/girepository-1.0" [
        pkgs.gtk4
        pkgs.glib
        pkgs.pango
        pkgs.gdk-pixbuf
      ]}''${GI_TYPELIB_PATH:+:$GI_TYPELIB_PATH}"
      export LD_LIBRARY_PATH="${lib.makeLibraryPath [
        pkgs.gtk4
        pkgs.glib
        pkgs.pango
        pkgs.gdk-pixbuf
      ]}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
      exec ${gtkPython}/bin/python3 ${./selector-sesiones.py}
    '';
  };

  # Este compositor temporal usa únicamente la pantalla integrada. Al conectarse
  # un monitor externo no crea un lienzo extendido entre ambas salidas.
  swayConfig = pkgs.writeText "korunix-selector-sway.conf" ''
    output eDP-1 enable
    output eDP-1 scale 1.5
    workspace 1 output eDP-1
    workspace 1

    input "NVTK0603:00 0603:F200" map_to_output eDP-1

    default_border none
    default_floating_border none
    exec ${selector}/bin/korunix-session-selector
  '';

  selectorSession = pkgs.runCommand "korunix-selector-session" {
    passthru.providedSessions = [ "korunix-selector" ];
  } ''
    mkdir -p "$out/share/wayland-sessions"
    cat > "$out/share/wayland-sessions/korunix-selector.desktop" <<EOF
    [Desktop Entry]
    Name=Korunix Session Selector
    Comment=Choose GNOME, Niri, or Gaming Mode
    Exec=${pkgs.sway}/bin/sway --config ${swayConfig}
    Type=Application
    DesktopNames=Korunix
    EOF
  '';
in
{
  # Registra una sesión Wayland independiente que Jovian puede abrir al salir
  # temporalmente de Gamescope.
  services.displayManager.sessionPackages = [ selectorSession ];
  environment.systemPackages = [ selectorSession ];

  # El menú "Cambiar a escritorio" de Gaming Mode abrirá el selector.

  # Necesario para que el gestor vuelva a iniciar la sesión escogida sin pasar
  # por un formulario de contraseña entre una sesión y otra.
  services.displayManager.sddm.autoLogin.relogin = lib.mkForce true;
}
