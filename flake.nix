{
  description = "TinyWiiBackupManager flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
      mkPkgs = system: import nixpkgs { inherit system; };
      runtimeLibsFor =
        pkgs: with pkgs; [
          libGL
          libxcb
          libxkbcommon
          libx11
          libxcursor
          libxext
          libxi
          libxinerama
          libxrandr
          vulkan-loader
          wayland
        ];
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          lib = pkgs.lib;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./assets
              ./build.rs
              ./Cargo.lock
              ./Cargo.toml
              ./package
              ./src
            ];
          };
          runtimeLibs = runtimeLibsFor pkgs;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "TinyWiiBackupManager";
            version = "5.1.23";

            inherit src;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            nativeBuildInputs = with pkgs; [
              makeWrapper
              pkg-config
            ];

            buildInputs =
              runtimeLibs
              ++ (with pkgs; [
                openssl
              ]);

            doCheck = false;

            postInstall = ''
              install -Dm644 package/linux/usr/share/applications/it.mq1.TinyWiiBackupManager.desktop \
                $out/share/applications/it.mq1.TinyWiiBackupManager.desktop
              install -Dm644 package/linux/usr/share/metainfo/it.mq1.TinyWiiBackupManager.metainfo.xml \
                $out/share/metainfo/it.mq1.TinyWiiBackupManager.metainfo.xml

              mkdir -p $out/share/icons
              cp -r package/linux/usr/share/icons/hicolor $out/share/icons/

              wrapProgram $out/bin/TinyWiiBackupManager \
                --prefix PATH : ${lib.makeBinPath [ pkgs.xdg-utils ]} \
                --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs}
            '';

            meta = with lib; {
              description = "A tiny game backup and homebrew app manager for the Wii";
              homepage = "https://github.com/mq1/TinyWiiBackupManager";
              license = licenses.gpl3Only;
              mainProgram = "TinyWiiBackupManager";
              platforms = platforms.linux;
            };
          };
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/TinyWiiBackupManager";
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          runtimeLibs = runtimeLibsFor pkgs;
        in
        {
          default = pkgs.mkShell {
            packages =
              runtimeLibs
              ++ (with pkgs; [
                cargo
                clippy
                openssl
                pkg-config
                rustc
                rustfmt
              ]);
          };
        }
      );
    };
}
