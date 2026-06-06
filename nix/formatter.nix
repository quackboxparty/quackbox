{ inputs, pkgs, ... }:
(inputs.treefmt-nix.lib.evalModule pkgs {
  projectRootFile = "flake.nix";
  programs = {
    nixfmt.enable = true;
    nixfmt.package = pkgs.nixfmt;

    statix.enable = true;
    deadnix.enable = true;

    prettier.enable = true;
  };

  settings.formatter = {
    statix.priority = 1;
    deadnix.priority = 2;
    nixfmt.priority = 3;
  };
}).config.build.wrapper
