{ pkgs, ... }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    nodejs_26
    pnpm_11
    playwright-driver.browsers

    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    bacon

    pkg-config
  ];

  PLAYWRIGHT_NODEJS_PATH = "${pkgs.nodejs}/bin/node";
  PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
  PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = 1;
  PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
  PLAYWRIGHT_HOST_PLATFORM_OVERRIDE = "ubuntu-24.04";

  shellHook = ''
    echo "dev env started"
  '';
}
