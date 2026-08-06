{...}: {
  programs.noctalia.settings = {
    theme = {
      mode = "auto";
      source = "community";
      builtin = "Noctalia";
      community_palette = "Everforest";
      templates_builtin = true;
      templates_community = true;
    };

    templates = {
      alacritty.target = ".config/alacritty/colors.toml";
      niri.target = ".config/niri/noctalia.kdl";
      vscode.target = ".config/Code/User/noctalia-colors.json";
      steam.target = ".config/millennium/themes/Noctalia/skin.css";
      discord.target = ".config/vesktop/themes/noctalia.css";
      gtk.target = ".config/gtk-3.0/colors.css";
    };
  };
}
