{
  description = "STM32F103C8 Embassy Project";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, naersk, rust-overlay }:
    let
      # Set the system architecture
      system = "x86_64-linux"; # Adjust as necessary
      pkgs = import nixpkgs {
        inherit system;
        overlays =
          [ (import rust-overlay) ]; # Import rust-overlay as a function
      };
      rustChannel = pkgs.rust-bin.stable.latest.default;

    in {

      # Define the development shell
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [ pkgs.pkg-config rustChannel pkgs.rustup ];
      };
    };
}
