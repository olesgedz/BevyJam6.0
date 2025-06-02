{
  description = "Playing with the bevy game engine";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    #nixpkgs.url = "nixpkgs/nixos-24.11";
  };
  nixConfig = {
    bash-prompt = ''\[\033[1;32m\][\[\e]0;\u@\h: \w\a\]dev-shell:\w]\$\[\033[0m\] '';
  };

  outputs = { self, nixpkgs }:
  let system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      bevy-deps = with pkgs; [
        udev alsa-lib vulkan-loader
        libxkbcommon
        # To use the x11 feature
        xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
        # To use wayland
        # libxkbcommon wayland
      ];
  in {

    devShells.x86_64-linux.default = pkgs.mkShell {
      shellHook = ''
        alias trunk="~/.cargo/bin/trunk"
      '';
      nativeBuildInputs = with pkgs; [
        # alsa-lib
        xorg.libX11
        pkg-config
      ];
      buildInputs = with pkgs; [
        # openssl
        # openssl.dev
        # Lightweight image viewer
        feh
        llvmPackages_17.clangUseLLVM
        llvmPackages_17.bintools
        llvmPackages_17.lldb
        rustup
      ] ++ bevy-deps;
      LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_17.libclang.lib ];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ((with pkgs; [
        # llvmPackages_17.clangUseLLVM
        # llvmPackages_17.bintools
        # xorg.libX11
        # xorg.libXcursor
        # xorg.libXrandr
        # xorg.libXi
        # pkgs.openssl
      ]) ++ bevy-deps);
      src = [
        ./flake.nix
        ./flake.lock
      ];
    };
  };
}
