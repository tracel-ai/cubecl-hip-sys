{
  description = "ROCm flake for NixOS.";

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";

  };

  nixConfig = {
    extra-substituters = [
      "https://devenv.cachix.org"
    ];
    extra-trusted-public-keys = [
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw="
    ];
  };

  outputs =
    {
      self,
      nixpkgs,
      devenv,
      systems,
      ...
    }@inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true; # Not needed today, maybe tomorrow!
          };
          lib = pkgs.lib;
          rocm = pkgs.rocmPackages;

        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              {
                languages.rust = {
                  enable = true;
                  # channel = "stable";
                  mold.enable = true;
                };

                # https://devenv.sh/reference/options/
                packages = [
                  rocm.clr
                  rocm.rocblas
                  rocm.rocm-smi
                ];

                enterShell = ''
                  export HIP_PATH=${rocm.clr}
                  export ROCM_PATH=${rocm.clr}

                  export EXTRA_LDFLAGS="-L${rocm.clr}/lib"
                  export EXTRA_CCFLAGS="-I${rocm.clr}/include"
                  # export LD_LIBRARY_PATH="${rocm.clr}/lib:${pkgs.ncurses5}/lib:$LD_LIBRARY_PATH"

                  # Helps if the device is either not supported or not found.
                  # export HSA_OVERRIDE_GFX_VERSION=10.3.0 

                  echo This flake uses the newest version of ROCm in nixos unstable.
                  echo You can override all rocm packages above.
                  echo
                '';

                env = {
                  LD_LIBRARY_PATH = lib.makeLibraryPath ([
                    rocm.clr
                    pkgs.ncurses5
                  ]);
                  RUST_LOG = "info";
                };

              }
            ];
          };
        }
      );
    };
}
