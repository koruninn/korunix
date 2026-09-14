{ ... }:
{
  # Umbriel declara su portal y mantenemos explícita su preferencia de escritorio.
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
