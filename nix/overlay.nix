{inputs, ...}: {
  flake.overlays.default = final: prev: {
    valfisk = prev.callPackage ./derivation.nix {
      /*
      the packages in this flake will use the pure `naersk.lib`,
      while users consuming this overlay directly will only fallback to it
      (or a new derivation) when needed.
      */
      naersk =
        final.naersk
        or inputs.naersk.lib.${prev.stdenv.hostPlatform.system}
        or (prev.callPackage inputs.naersk {});

      inherit
        ((final.darwin or prev.darwin).apple_sdk.frameworks)
        CoreFoundation
        Security
        SystemConfiguration
        ;

      inherit (final.darwin or prev.darwin) IOKit;
    };
  };
}
