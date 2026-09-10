{
  config,
  pkgs,
  ...
}: {
  imports = [    
    ./escritorios
    ./greeter
  ];
}
