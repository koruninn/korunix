{
  config,
  pkgs,
  ...
}: {
  nix.settings = {
    extra-substituters = [
      "https://noctalia.cachix.org"
      "https://ezkea.cachix.org" # <--- Añadido
    ];
    extra-trusted-public-keys = [
      "noctalia.cachix.org-1:pCOR47nnMEo5thcxNDtzWpOxNFQsBRglJzxWPp3dkU4="
      "ezkea.cachix.org-1:io85OCXmr5WwSZQYw7066RA2fNdOeOwGEgMDwiDxUCg=" # <--- Añadido
    ];

    experimental-features = ["nix-command" "flakes"];
  };
}
