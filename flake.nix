{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs { inherit system overlays; };
        libPath = pkgs.lib.makeLibraryPath (with pkgs; [
          vulkan-loader
          libGL libGLU
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          mimalloc
        ]);
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            pkg-config
            rust-bin.stable.latest.default
            rust-analyzer
            bacon
            cargo-expand
            udev
            gdb
            linuxKernel.packages.linux_zen.perf
            mold
            cargo-flamegraph
            hotspot
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = libPath;
        };
      });
}
