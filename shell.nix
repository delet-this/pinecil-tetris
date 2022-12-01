{ pkgs, system, crossPkgs }:

with builtins;
let
  inherit (pkgs) stdenv lib;
  inherit crossPkgs;

  # moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  # pkgs = nixpkgs.legacyPackages.x86_64-linux;
  # pkgs.overlays = [ moz_overlay ];

  # Get a custom cross-compile capable Rust install of a specific channel and
  # build. Tock expects a specific version of Rust with a selection of targets
  # and components to be present.
  rustBuild = (
    pkgs.rustChannelOf (
      let
        # Read the ./rust-toolchain (and trim whitespace) so we can extrapolate
        # the channel and date information. This makes it more convenient to
        # update the Rust toolchain used.
        rustToolchain = builtins.replaceStrings ["\n" "\r" " " "\t"] ["" "" "" ""] (
          builtins.readFile ./rust-toolchain
        );
      in
        {
          channel = lib.head (lib.splitString "-" rustToolchain);
          date = lib.concatStringsSep "-" (lib.tail (lib.splitString "-" rustToolchain));
        }
    )
  ).rust.override {
    targets = [
      # "thumbv7em-none-eabi" "thumbv7em-none-eabihf" "thumbv6m-none-eabi"
      # "riscv32imc-unknown-none-elf" "riscv32i-unknown-none-elf"
      "riscv32imac-unknown-none-elf"
    ];
    extensions = [
      "rust-src" # required to compile the core library
      "llvm-tools-preview" # currently required to support recently added flags
    ];
  };

  # riscv-toolchain = import pkgs {
  #   # localSystem = "${system}";
  #   crossSystem = { config = "riscv32--none-elf"; };
  # };

in
  pkgs.mkShell {
    name = "pinecil-tetris";

    buildInputs = with pkgs; [
      # --- Toolchains ---
      rustBuild 
      # pkgs.pkgsCross.riscv32.clang
      # pkgs.pkgsCross.riscv32.mold

      # --- Convenience and support packages ---
      # python3Full

      # Required for tools/print_tock_memory_usage.py
      # pythonPackages.cxxfilt


      # --- CI support packages ---
      qemu
      # crossPkgs.pkgsHostTarget.binutils
      # pkgs.pkgsCross.riscv32.binutils
      cargo-binutils # for objcopy
      dfu-util
    ];

    LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";

    # The defaults "objcopy" and "objdump" are wrong (stem from the standard
    # environment for x86), use "llvm-obj{copy,dump}" as defined in the makefile
    shellHook = ''
      unset OBJCOPY
      unset OBJDUMP
    '';
  }

