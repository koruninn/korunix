{
  config,
  pkgs,
  ...
}: {
  # Habilitar el servicio de impresión CUPS
  services.printing.enable = true;

  # Añadir los drivers de Epson
  services.printing.drivers = with pkgs; [
    epson_201207w
  ];

  hardware.sane.enable = true;
}
