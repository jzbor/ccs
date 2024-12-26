{
  description = "Rust implementation of the Calculus of Communicating Systems";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    cf.url = "github:jzbor/cornflakes";
  };

  outputs = { self, nixpkgs, cf, crane, ... }: (cf.mkLib nixpkgs).flakeForDefaultSystems (system:
  let
    inherit (nixpkgs) lib;
    pkgs = nixpkgs.legacyPackages.${system};
    craneLib = crane.mkLib pkgs;
    benchmarkPythonEnv = pkgs.python3.withPackages(ps: [ ps.matplotlib ]);
  in {
    packages = rec {
      default = ccs;

      ccs = craneLib.buildPackage {
        src = ./.;
        nativeBuildInputs = [ pkgs.makeWrapper ];
        postInstall = ''
          wrapProgram $out/bin/ccs --prefix PATH : ${lib.makeBinPath [ pkgs.graphviz ]}
        '';
      };

      benchmark = pkgs.writeShellApplication {
        name = "benchmark";
        text = "${benchmarkPythonEnv}/bin/python3 ${self}/benchmark.py ${ccs}/bin/ccs \"$@\"";
      };

      render-benchmark = pkgs.writeShellApplication {
        name = "render-benchmark";
        text = "${benchmarkPythonEnv}/bin/python3 ${self}/render_benchmark.py \"$@\"";
      };


      profile = let
        ccs = craneLib.buildPackage {
          src = ./.;
          CARGO_PROFILE = "profiling";
        };
      in pkgs.writeShellApplication {
        name = "profile";
        runtimeInputs = with pkgs; [
          linuxPackages_latest.perf
        ];
        text = ''
          perf record -g -F 99 --call-graph=dwarf ${ccs}/bin/ccs "$@"
          perf script -F +pid > profile.perf
          rm perf.data
          echo "output: $(pwd)/profile.perf"
        '';
      };
    };

    devShells.default = craneLib.devShell {
      inherit (self.packages.${system}.default) name;

      # Additional tools
      nativeBuildInputs = [
        benchmarkPythonEnv
      ];
    };
  });
}
