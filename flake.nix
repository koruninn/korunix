{
  description = "Korunix";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "nixpkgs/nixos-26.05";

    # Perfil de la ROG Ally original (RC71L).
    nixos-hardware = {
      url = "github:NixOS/nixos-hardware";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    noctalia = {
      url = "github:noctalia-dev/noctalia/";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Luciole necesita una versión de Glycin más reciente.
    luciole.url = "github:noctalia-dev/luciole/7afafc2ee86be3786955ab006a4b0da79ab219d5";

    umbriel = { url = "github:noctalia-dev/umbriel/e65cc65ecec383528715d4bb50747b9f46199ab7"; };
    umbriel-src = {
      url = "github:noctalia-dev/umbriel";
      flake = false;
    };

    # Teclado virtual.
    sysboard-src = {
      url = "github:System64fumo/sysboard";
      flake = false;
    };

    # Soporte para el lector de huellas de la Ally.
    libfprint-egismoc-sdcp = {
      url = "github:TenSeventy7/libfprint-egismoc-sdcp/4d128d4f6f0b46182572126e84df88a73ac27859";
      flake = false;
    };

    # Parches para el sensor 1c7a:0588.
    ally-fingerprint-patches = {
      url = "github:podothecat/decky-fingerprint-unlock/d5b077c3cfad098909a2849529a27aff55e5354f";
      flake = false;
    };

    # Lanzadores de juegos de HoYoverse.
    aagl.url = "github:ezKEa/aagl-gtk-on-nix";
    aagl.inputs.nixpkgs.follows = "nixpkgs";

    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    figma-linux-next.url = "github:arximus88/figma-linux-next";

    tether.url = "github:zackb/tether/3b14d96104a958673f88e5c70d1481f810663052";
    tether.inputs.nixpkgs.follows = "nixpkgs";

    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";
  };

  outputs = {
    nixpkgs,
    nixpkgs-stable,
    ...
  } @ inputs: let
    equipo = import ./equipo.nix;

    nixpkgsSeleccionado =
      if equipo.canal == "stable" then nixpkgs-stable
      else if equipo.canal == "unstable" then nixpkgs
      else throw "Canal no válido en equipo.nix: ${equipo.canal}. Usa stable o unstable.";

    system = equipo.arquitectura;
    lib = nixpkgsSeleccionado.lib;
  in {
    formatter = nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"] (
      arquitectura: nixpkgs.legacyPackages.${arquitectura}.alejandra
    );

    nixosConfigurations.veskalia = lib.nixosSystem {
      inherit system;
      specialArgs = {
        inherit inputs equipo;
        canal = equipo.canal;
      };
      modules = [
        ./configuracion.nix
        ./hardware.nix
        ./usuario.nix
        ./modulos/base
        ./aplicaciones
        ./apariencia
        ./servicios
        {
          networking.hostName = "veskalia";
        }
      ];
    };
  };
}
