{ pkgs, ... }:
pkgs.stdenvNoCC.mkDerivation rec {
  name = "quackbox";
  src = ../.;
  nativeBuildInputs = with pkgs; [
    nodejs_24
    pnpm_11
    pnpmConfigHook
  ];
  pnpmDeps = pkgs.fetchPnpmDeps {
    inherit src;
    inherit (pkgs) pnpm;
    pname = name;
    fetcherVersion = 2;
    hash = "sha256-ZYmk1WY/v/sGj7rQL19pMs8UOzJQNhrlDUJ8jgxIgPw=";
  };
  buildPhase = ''
    pnpm build
  '';
  installPhase = ''
    cp -r dist $out
  '';
}
