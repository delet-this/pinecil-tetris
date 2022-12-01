{
  description = "pinecil-tetris";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs = { self, nixpkgs, mozilla }: 
  let 
    moz_overlay = mozilla.overlay;
    pkgs = import nixpkgs { overlays = [moz_overlay]; };
    crossPkgs = import nixpkgs { crossSystem = { config = "riscv32-none-elf"; }; };
    system = "x86_64-linux";
    # pkgs = mynixpkgs.legacyPackages.x86_64-linux; 
  in
  {
    devShell.x86_64-linux = import ./shell.nix { inherit pkgs; inherit system; inherit crossPkgs; };
  };
}
