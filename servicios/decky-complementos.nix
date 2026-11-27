{
  # Complementos que uso en el modo consola.
  plugins = [
    { carpeta = "Bluetooth"; nombre = "Bluetooth"; autor = "Outpox"; version = "3.1.1"; }
    { carpeta = "decky-desktop-apps"; nombre = "Desktop Apps"; autor = "Kirill Nikiforov"; version = "0.1.0"; }
    { carpeta = "decky-speed-test"; nombre = "Speed Test"; autor = "Chris Garson"; version = "1.0.2"; }
    { carpeta = "decky-steamgriddb"; nombre = "SteamGridDB"; autor = "SteamGridDB"; version = "1.8.1"; }
    { carpeta = "free-loader"; nombre = "Free Loader"; autor = "jwhitlow45"; version = "1.6.0"; }
    { carpeta = "HueSync"; nombre = "HueSync"; autor = "honjow"; version = "3.8.0"; }
    { carpeta = "isthereanydeal-for-deck"; nombre = "IsThereAnyDeal for Deck"; autor = "JtdeGraaf"; version = "1.1.1"; }
    { carpeta = "SDH-AnimationChanger"; nombre = "Animation Changer"; autor = "Justin Marentette"; version = "1.3.2"; }
    { carpeta = "SDH-CssLoader"; nombre = "CSS Loader"; autor = "DeckThemes"; version = "2.1.3"; }
  ];

  # Temas de CSS Loader y mis ajustes iniciales.
  temas = [
    {
      carpeta = "Better Achievements";
      id = "ea439a7f-1205-4d17-bdf9-baf1f8208bc8";
      version = "v1.2";
      ajustes = {
        active = true;
        "Rare Shine Animation" = "Only in achievements page";
        "Achievement Icon Radius" = "0px";
      };
    }
    {
      carpeta = "Better Game Icons";
      id = "a4e9c87b-5201-4ba3-b7ef-204fdd934b53";
      version = "v1.4";
      ajustes = {
        active = true;
        "Radius" = "0px";
        "Downloading Icon Background " = "Transparent";
      };
    }
    {
      carpeta = "Deck Verified Badge";
      id = "9bbfc3b7-7baf-42fa-acbb-80d238a07912";
      version = "v1.3";
      ajustes = {
        active = true;
        "Icon Style" = "SteamOS";
      };
    }
    {
      carpeta = "Float";
      id = "1ee17154-c237-42d9-a910-dbe92303950d";
      version = "v1.0.5";
      ajustes = {
        active = true;
        "Home layout" = "Banner";
        "Shelf blur" = "Medium";
        "Keyboard" = "Yes";
      };
    }
    {
      carpeta = "Game Cover Reflections";
      id = "26e6a114-4513-4440-9e81-65460ce54884";
      version = "v1.0";
      ajustes = {
        active = true;
        "Focused Opacity" = "0.5";
        "Unfocused Opacity" = "0.3";
        "Blur" = "3";
        "Opacity Smoothness" = "0.5";
      };
    }
    {
      carpeta = "Game Title in Library";
      id = "35fc4ae2-c938-4494-afaf-7586ad359db3";
      version = "v1.0.2";
      ajustes = {
        active = true;
        "Grid Row Gap" = "4";
      };
    }
    {
      carpeta = "Hide Volume Label";
      id = "64cea947-b873-4fd0-9947-35a33b3dfb51";
      version = "v1.0";
      ajustes = {
        active = true;
      };
    }
    {
      carpeta = "Show Download Status";
      id = "2bf7d0cf-5bc0-40d0-8b26-2dd286505994";
      version = "v1.4.2";
      ajustes = {
        active = true;
        "Varients" = "Original";
      };
    }
    {
      carpeta = "Tilted Home";
      id = "231c969d-b16f-41a0-98a1-cec8aeb557ba";
      version = "v1.3";
      ajustes = {
        active = true;
        "Angle" = "-5";
        "Tilt Method" = "3D";
      };
    }
    {
      carpeta = "handheld-controller-glyphs";
      id = "27d2dfb0-f58c-468f-81d5-f06534534133";
      version = "v2.1";
      ajustes = {
        active = true;
        "Handheld" = "ASUS ROG Ally";
        "Hide buttons and features that the device doesn't have" = "Yes";
        "Swap View/Menu with Guide/QAM" = "No";
        "[ROG Xbox Ally] Swap Xbox with Armory Crate" = "No";
        "[Lenovo Legion Go] Swap Guide with View/Menu" = "View";
      };
    }
  ];
}
