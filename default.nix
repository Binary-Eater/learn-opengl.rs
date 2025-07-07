{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clang
    # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
    llvmPackages_16.bintools
    rustup
    pkg-config
    openssl

    # For glfw-sys
    # This overrideAttrs is because of how FindX11.cmake is defined
    # https://github.com/Kitware/CMake/blob/master/Modules/FindX11.cmake
    # This builds cmake from source....
    (cmake.overrideAttrs (final: prev: {
      # TODO grab version and clip off patch version number
      postFixup = prev.postFixup + "\n" + lib.concatStringsSep "\n" [ 
        "sed -i 's|/var/empty/X11/lib|${pkgs.xorg.libX11.dev}/include|' $out/share/cmake-3.30/Modules/FindX11.cmake"
        "sed -i 's|/var/empty/X11/lib|${pkgs.xorg.libX11.out}/lib|' $out/share/cmake-3.30/Modules/FindX11.cmake"
      ];
    }))
  ];

  # For glfw-sys
  X11_X11_INCLUDE_PATH = "${pkgs.xorg.libX11.dev}/include";
  X11_X11_LIB = "${pkgs.xorg.libX11.out}/lib/libX11.so";

  # Cannot be used by glfw-sys as-is without static linking Nix overlay
  C_INCLUDE_PATH = "${pkgs.xorg.libX11.dev}/include:${pkgs.glfw}/include";
  LIBRARY_PATH = "${pkgs.xorg.libX11.out}/lib:${pkgs.glfw}/lib";
  PKG_CONFIG_PATH = "${pkgs.glfw}/lib/pkgconfig";

  OPENSSL_DEV = pkgs.openssl.dev;
  RUSTC_VERSION = "stable-2025-06-26";
  # https://github.com/rust-lang/rust-bindgen#environment-variables
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  shellHook = ''
      export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
      export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
      '';
  # Add precompiled library to rustc search path
  RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
    # add libraries here (e.g. pkgs.libvmi)
  ]);
  # Add glibc, clang, glib and other headers to bindgen search path
  BINDGEN_EXTRA_CLANG_ARGS =
    # Includes with normal include path
    (builtins.map (a: ''-I"${a}/include"'') [
      # add dev libraries here (e.g. pkgs.libvmi.dev)
      pkgs.glibc.dev
    ])
    # Includes with special directory paths
    ++ [
      ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
      ''-I"${pkgs.glib.dev}/include/glib-2.0"''
      ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
    ];
}
