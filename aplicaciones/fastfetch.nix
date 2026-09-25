{ ... }: {
  environment.etc."fastfetch/config.jsonc".text = builtins.toJSON {

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
        format = "{2}";
      }
      {
        type = "kernel";
        key = "Núcleo";
      }
      {
        type = "shell";
        key = "Shell";
      }
      {
        type = "wm";
        key = "Escritorio";
        format = "{2}";
      }
      {
        type = "cpu";
        key = "Procesador";
        format = "{1}";
      }
      {
        type = "memory";
        key = "Memoria RAM";
        format = "{used} / {total}";
      }
      {
        type = "disk";
        key = "Disco (/)";
        format = "{size-free} / {size-total}";
      }
      "break"
      "colors"
    ];
  };
}
