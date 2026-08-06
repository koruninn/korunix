{...}: {
  programs.fastfetch = {
    enable = true;
    settings = {
      "$schema" = "https://github.com/fastfetch-cli/fastfetch/raw/master/doc/json_schema.json";

      display = {
        key = {
          width = 15;
        };
        size = {
          binaryPrefix = "jedec";
        };
        separator = " ➜ ";
      };

      logo = {
        source = "nixos_small";
        padding = {
          right = 3;
          left = 2;
        };
      };

      modules = [
        "break"
        {
          type = "os";
          key = "SO";
          keyColor = "green";
          format = "{2}";
        }
        {
          type = "kernel";
          key = "Núcleo";
          keyColor = "yellow";
        }
        {
          type = "shell";
          key = "Shell";
          keyColor = "magenta";
        }
        {
          type = "wm";
          key = "Escritorio";
          keyColor = "blue";
          format = "{2}";
        }
        {
          type = "cpu";
          key = "Procesador";
          keyColor = "yellow";
          format = "{1}";
        }
        {
          type = "memory";
          key = "Memoria RAM";
          keyColor = "magenta";
          format = "{used} / {total}";
        }
        {
          type = "disk";
          key = "Disco (/)";
          keyColor = "cyan";
          folders = "/";
          format = "{size-used} / {size-total}";
        }
        "break"
        "colors"
      ];
    };
  };
}
