self:
{ config
, lib
, pkgs
, ...
}:

let
  cfg = config.programs.aicentre;
in
{
  options.programs.aicentre = {
    enable = lib.mkEnableOption "aicentre, the command line tool for P1 compute";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.aicentre;
      defaultText = lib.literalExpression "inputs.p1-compute.packages.\${system}.aicentre";
      description = "The aicentre package to install.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
