{
  config,
  pkgs,
  ...
}: let
  fetchConfig = pkgs.writeText "korunix-fetch-config" ''
    # Misma selección y orden de información que Fastfetch en Korunix.
    os
    kernel
    shell
    wm
    cpu
    memory
    disk
    colors

    # Apariencia del panel de información.
    label_color=magenta

    # Animación 3D.
    spin=xy
    speed=1.0
    size=1.0
    light=top-left
    v_alignment=top
    h_alignment=left
  '';
in {
  environment.systemPackages = [
    pkgs.fetch
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
