{...}: {
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
