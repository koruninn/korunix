{
  config,
  pkgs,
  ...
}: let
  korunixFetch = pkgs.fetch.overrideAttrs (old: {
    postPatch = (old.postPatch or "") + ''
      # Un lienzo más estrecho acerca el logo al bloque de información.
      substituteInPlace fetch.c \
        --replace-fail '#define ANIM_WIDTH 60' '#define ANIM_WIDTH 18'

      # Presentación compacta: icono, flecha y valor; sin cabecera usuario@equipo.
      substituteInPlace fetch.c \
        --replace-fail 'snprintf(line, sizeof(line), "\033[1;%sm%s\033[0m: %s", label_color, label,' 'snprintf(line, sizeof(line), "\033[1;%sm%s\033[0m  ➜  %s", label_color, label,'
      sed -i '/static void gather_title(void) {/a\  return;' fetch.c

      # Etiquetas simbólicas con Nerd Fonts.
      sed -i \
        -e 's/add_info("OS",/add_info("",/g' \
        -e 's/add_info("Kernel",/add_info("",/g' \
        -e 's/add_info("Shell",/add_info("",/g' \
        -e 's/add_info("WM",/add_info("",/g' \
        -e 's/add_info("CPU",/add_info("",/g' \
        -e 's/add_info("Memory",/add_info("󰍛",/g' \
        -e 's/"Disk (%s)"/""/g' \
        fetch.c

      # Dejamos solo la información útil, como en el Fastfetch de Korunix.
      substituteInPlace fetch.c \
        --replace-fail 'add_info("", "%s %s", pretty, u.machine);' 'add_info("", "NixOS");' \
        --replace-fail 'add_info("", "%s %s%s", wm, version, is_wayland ? " (Wayland)" : "");' 'add_info("", "%s", wm);' \
        --replace-fail 'add_info("", "%s%s", wm, is_wayland ? " (Wayland)" : "");' 'add_info("", "%s", wm);' \
        --replace-fail 'add_info("", "%s (%d) @ %.2f GHz", name, cores, ghz);' 'add_info("", "%s", name);' \
        --replace-fail 'add_info("", "%s (%d) @ %.2f GHz", name, cores, max_ghz);' 'add_info("", "%s", name);' \
        --replace-fail 'add_info("", "%s (%d)", name, cores);' 'add_info("", "%s", name);'

      # En algunos Ryzen con gráfica integrada /proc/cpuinfo añade este sufijo.
      # Lo quitamos para mostrar únicamente el modelo del procesador.
      sed -i '/if (name\[0\]) {/i\  char *radeon_suffix = strstr(name, " with Radeon Graphics");\n  if (radeon_suffix)\n    *radeon_suffix = 0;' fetch.c

      # RAM y disco: usado / total, sin porcentaje ni tipo de sistema de archivos.
      substituteInPlace fetch.c \
        --replace-warn '%.2f GiB / %.2f GiB (\033[%sm%d%%\033[0m) - %s' '%.2f GiB / %.2f GiB' \
        --replace-warn '%.2f GiB / %.2f GiB (\033[%sm%d%%\033[0m)' '%.2f GiB / %.2f GiB'
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

    # Conservamos la silueta de NixOS: menos relieve, bloques sólidos y giro
    # sobre un solo eje para que el copo siga siendo reconocible al animarse.
    shading_mode=blocks
    spin=y
    speed=0.75
    size=1.80
    depth=0.30
    height=14
    light=front
    v_alignment=center
    h_alignment=left
  '';
in {
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
