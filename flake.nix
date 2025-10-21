{
  description = "Flake utils demo";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        toolchain = fenix.packages.${system}.default.toolchain;
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            toolchain
            rust-analyzer
            wayland
            wayland-protocols
            libxkbcommon
            alsa-lib
            udev
            glfw-wayland
            vulkan-loader
            mesa
          ];
        };
        
        packages.default = pkgs.buildFHSEnv {
          name = "electrical-bboard-fhs";
          targetPkgs = pkgs: [
            (self.packages.${system}.electrical-bboard-unwrapped)
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.alsa-lib
            pkgs.udev
            pkgs.glfw-wayland
            pkgs.vulkan-loader
						pkgs.mesa
          ];
          runScript = "electrical-bboard"; 
        };
        
        packages.electrical-bboard-unwrapped = (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
          rustfmt = toolchain;
        }).buildRustPackage {
          pname = "electrical-bboard";
          version = "0.1.0";
          src = ./.;
          rpath = true;
          cargoLock.lockFile = ./Cargo.lock;
					nativeBuildInputs = with pkgs; [
            toolchain
	          pkg-config
						wayland
						wayland-protocols
						alsa-lib
						udev
          ];
         	propagatedBuildInputs = with pkgs; [
            openssl
		        wayland
						wayland-protocols
						alsa-lib
						udev
            glfw-wayland
            xorg.libxkbfile
            libxkbcommon
					];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.alsa-lib
            pkgs.udev
            pkgs.wayland
            pkgs.wayland-protocols
          ];
          postFixup = ''
                    lib_path="${pkgs.lib.makeLibraryPath [
                                             pkgs.wayland
                                             pkgs.wayland-protocols
                                             pkgs.alsa-lib
                                             pkgs.udev
                                             pkgs.libxkbcommon
                                             pkgs.glfw-wayland
                                             pkgs.xorg.libxkbfile
                                           ]}"
                    patchelf --set-rpath $LD_LIBRARY_PATH $out/bin/electrical-bboard
          '';        
					postInstall = ''
											cp -r assets $out/bin/assets
					'';
        };
      }
    );
}
