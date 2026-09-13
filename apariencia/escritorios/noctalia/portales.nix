{ ... }:
{
  # Niri ya declara su selección de portales desde el módulo oficial de NixOS.
  # Umbriel y Hyprland también instalan sus portales, pero ambos módulos aportan
  # configuración mediante configPackages. Al convivir los tres compositores,
  # declaramos aquí las preferencias por escritorio para que una sesión no use
  # accidentalmente el portal de captura de otra.
  xdg.portal.config = {
    hyprland = {
      default = [
        "hyprland"
        "gtk"
      ];
      "org.freedesktop.impl.portal.Secret" = "gnome-keyring";
    };

    umbriel = {
      default = [
        "umbriel"
        "gtk"
      ];
      "org.freedesktop.impl.portal.Secret" = "gnome-keyring";
    };
  };
}
