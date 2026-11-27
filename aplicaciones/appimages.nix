{
  # AppImages que guarda NixDeck en la microSD.
  nixdeck = [
    {
      nombre = "DuckStation";
      repo = "stenzek/duckstation";
      version = "tags/latest";
      archivo = "DuckStation-x64.AppImage";
    }
    {
      nombre = "Xenia";
      repo = "xenia-canary/xenia-canary";
      version = "latest";
      archivo = "xenia_canary_linux.AppImage";
    }
  ];

  # AppImages integrados con Gear Lever y sus fuentes de actualización.
  gearlever = [
  ];
}
