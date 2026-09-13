{...}: {
  nix.settings = {
    extra-substituters = [
      "https://noctalia.cachix.org"
      "https://ezkea.cachix.org"
    ];
    extra-trusted-public-keys = [
      "noctalia.cachix.org-1:pCOR47nnMEo5thcxNDtsWpOxNFQsBRglJzxWPp3dkU4="
      "ezkea.cachix.org-1:io85OCXmr5WwSZQYw7066RA2fNdOeOwGEgMDwiDxUCg="
    ];
  };
}
