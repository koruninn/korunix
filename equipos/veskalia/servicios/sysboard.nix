{ inputs, pkgs, ... }:
let
  sysboard = pkgs.stdenv.mkDerivation {
    pname = "sysboard";
    version = "unstable-${inputs.sysboard-src.shortRev or "source"}";
    src = inputs.sysboard-src;

    nativeBuildInputs = with pkgs; [
      gnumake
      makeWrapper
      pkg-config
      wayland-scanner
    ];

    buildInputs = with pkgs; [
      gtkmm4
      gtk4-layer-shell
      wayland
    ];

    # Sysboard asume rutas /usr; en Nix apuntamos a sus datos en el store.
    postPatch = ''
      substituteInPlace src/main.cpp \
        --replace-fail '"/usr/share/sys64/board/config.conf"' \
          "\"$out/share/sys64/board/config.conf\""

      substituteInPlace src/window.cpp \
        --replace-fail '"/usr/share/sys64/board/style.css"' \
          "\"$out/share/sys64/board/style.css\""
    '';

    # El tarball de un input flake no trae .git, que el Makefile espera.
    preBuild = ''
      cat > src/git_info.hpp <<'HEADER'
#define GIT_COMMIT_MESSAGE "Nix-locked source"
#define GIT_COMMIT_DATE "see flake.lock"
HEADER
    '';

    installPhase = ''
      runHook preInstall

      make install \
        PREFIX="$out" \
        BINDIR="$out/bin" \
        LIBDIR="$out/lib" \
        DATADIR="$out/share"

      # El ejecutable carga libsysboard.so mediante dlopen().
      wrapProgram "$out/bin/sysboard" \
        --prefix LD_LIBRARY_PATH : "$out/lib"

      runHook postInstall
    '';

    meta = {
      description = "Teclado virtual GTK4 para Wayland";
      homepage = "https://github.com/System64fumo/sysboard";
      license = pkgs.lib.licenses.gpl3Only;
      platforms = pkgs.lib.platforms.linux;
      mainProgram = "sysboard";
    };
  };

  sysboardToggle = pkgs.writeShellApplication {
    name = "korunix-sysboard-toggle";
    runtimeInputs = [ pkgs.coreutils pkgs.procps sysboard ];
    text = ''
      if pgrep -x sysboard >/dev/null 2>&1; then
        exec pkill -RTMIN -x sysboard
      fi

      # Normalmente ya está iniciado por Umbriel. Si se cerró, lo arrancamos
      # y esperamos a que registre el protocolo de teclado antes de alternarlo.
      sysboard &
      for attempt in 1 2 3 4 5 6 7 8 9 10; do
        if pgrep -x sysboard >/dev/null 2>&1; then
          sleep 0.3
          pkill -RTMIN -x sysboard && exit 0
        fi
        sleep 0.1
      done
      echo "No se pudo iniciar Sysboard." >&2
      exit 1
    '';
  };
in {
  environment.systemPackages = [ sysboard sysboardToggle ];
}
