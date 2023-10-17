{
  perSystem = {
    lib,
    pkgs,
    self',
    ...
  }: {
    legacyPackages = {
      valfisk-docker = pkgs.dockerTools.buildImage {
        name = "valfisk";
        tag = "latest";

        copyToRoot = [
          pkgs.dockerTools.caCertificates
        ];

        config.Cmd = [
          "${lib.getExe self'.legacyPackages.valfisk-static}"
        ];
      };
    };
  };
}
