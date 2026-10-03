{ ... }:
{
  # Aquí decimos qué portal debe usar Umbriel y dejamos claro este preferencia para las aplicaciones.
  xdg.portal.config = {
    umbriel = {
      default = [
        "umbriel"
        "gtk"
      ];
      "org.freedesktop.impl.portal.Secret" = "gnome-keyring";
    };
  };
}
