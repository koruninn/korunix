# Hardware y capacidades base administradas por Korunix.
#
# Este módulo contiene hechos de la máquina que Korunix necesita conocer para
# tomar decisiones automáticamente. No son preferencias que una persona deba
# aprender ni escoger durante el uso normal.
{
  config,
  lib,
  pkgs,
  ...
}: {
  options.korunix.hardware = {
    # UEFI o BIOS se detecta al adoptar el equipo. Se conserva declarativamente
    # porque Nix necesita conocerlo durante la construcción, cuando consultar
    # directamente /sys no sería una fuente reproducible.
    firmware = lib.mkOption {
      type = lib.types.enum [
        "uefi"
        "bios"
      ];

      description = "Tipo de firmware detectado y adoptado para este equipo.";
    };
  };

  config = lib.mkIf config.korunix.enable {
    # Los blobs redistribuibles permiten cargar firmware y microcódigo que el
    # hardware necesita sin convertirlos en elecciones manuales del usuario.
    hardware.enableRedistributableFirmware = true;

    # fwupd permite detectar firmware actualizable. Korunix nunca instala una
    # actualización de firmware sin mostrarla y pedir una acción explícita.
    services.fwupd.enable = true;

    # El refresco automático de metadatos de fwupd puede necesitar Polkit desde
    # un servicio no interactivo y hacer que una activación correcta de NixOS
    # termine informando un fallo. Korunix conserva fwupd, pero realizará esta
    # consulta explícitamente cuando la persona entre a la sección de firmware.
    systemd.services.fwupd-refresh.enable = false;
    systemd.timers.fwupd-refresh.enable = false;

    # El detector local utiliza PCI para identificar dispositivos y jq para
    # ofrecer el mismo modelo estructurado a terminal y a la GUI futura.
    environment.systemPackages = [
      pkgs.pciutils
      pkgs.jq
    ];
  };
}
