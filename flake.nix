{
  inputs.nixpkgs.url = "github:nixos/nixpkgs";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = {nixpkgs, rust-overlay, ...}: let
    system = "x86_64-linux";
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs { inherit system overlays; };
    packages = with pkgs; [

      (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)

      just

      pkg-config
      openssl

      fontconfig
      freetype
      libGL
      egl-wayland
      wayland

      cfitsio

      libX11
      libXcursor
      libXi
      libxkbcommon
      gtk3
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      inherit packages;
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
    };
  };
}
