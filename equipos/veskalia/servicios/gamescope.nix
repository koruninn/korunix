{ config, equipo, pkgs, ... }:
let
  usuario = config.users.users.${equipo.persona};
  script = pkgs.writeText "99-night-mode-test.lua"
    "gamescope.hook(\"OnPostPaint\", function()\n    gamescope.convars.composite_force.value = true\nend)\n";
  destino = "${usuario.home}/.config/gamescope/scripts/99-night-mode-test.lua";
in
{
  environment.etc."korunix/gamescope/99-night-mode-test.lua".source = script;

  system.activationScripts.korunixGamescopeNightMode.text =
    "mkdir -p ${usuario.home}/.config/gamescope/scripts\n"
    + "chown ${equipo.persona}:${usuario.group} ${usuario.home}/.config/gamescope ${usuario.home}/.config/gamescope/scripts\n"
    + "chmod 0755 ${usuario.home}/.config/gamescope ${usuario.home}/.config/gamescope/scripts\n"
    + "ln -sfn ${script} ${destino}\n"
    + "chown -h ${equipo.persona}:${usuario.group} ${destino}\n";
}
