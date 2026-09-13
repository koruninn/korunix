{
  config,
  pkgs,
  ...
}: let
  korunixFetch = pkgs.fetch.overrideAttrs (old: {
    postPatch = (old.postPatch or "") + ''
      # Compactamos el lienzo del logo para acercarlo a la información.
      sed -i 's/#define ANIM_WIDTH 60/#define ANIM_WIDTH 24/' fetch.c

      # Sustituimos las etiquetas de texto por iconos Nerd Font.
      sed -i \
        -e 's/add_info("OS",/add_info("",/g' \
        -e 's/add_info("Kernel",/add_info("",/g' \
        -e 's/add_info("Shell",/add_info("",/g' \
        -e 's/add_info("WM",/add_info("",/g' \
        -e 's/add_info("CPU",/add_info("",/g' \
        -e 's/add_info("Memory",/add_info("",/g' \
        -e 's/"Disk (%s)"/" (%s)"/g' \
        fetch.c

      # La cabecera usuario@equipo no aporta información útil en este panel.
      sed -i '/static void gather_title(void) {/a\  return;' fetch.c
    '';
  });

  fetchConfig = pkgs.writeText "korunix-fetch-config" ''
    os
    kernel
    shell
    wm
    cpu
    memory
    disk
    colors

    label_color=magenta

    # Logo algo mayor y centrado respecto al bloque de información.
    spin=xy
    speed=1.0
    size=1.35
    height=12
    light=top-left
    v_alignment=center
    h_alignment=left
  '';
in {
  # Símbolos usados como etiquetas por fetch.
  fonts.packages = [
    pkgs.nerd-fonts.symbols-only
  ];

  environment.systemPackages = [
    korunixFetch
  ];

  # fetch solo lee su configuración desde el directorio XDG del usuario.
  # La dejamos declarada mientras la configuración de persona se desacopla
  # del usuario fijo de Korunix en una revisión posterior.
  system.activationScripts.fetchConfig.text = ''
    install -d -m 0755 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      ${config.users.users.koru.home}/.config/fetch

    install -m 0644 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      ${fetchConfig} \
      ${config.users.users.koru.home}/.config/fetch/config
  '';
}
