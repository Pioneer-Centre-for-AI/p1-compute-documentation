self:
{ config
, lib
, pkgs
, ...
}:

let
  cfg = config.programs.aicentre;
  toml = pkgs.formats.toml { };
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

    settings = lib.mkOption {
      inherit (toml) type;
      default = { };
      example = lib.literalExpression ''
        {
          dtu.user = "s123456";
          gefion.user = "jdoe";  # jobs are charged to P1's iu_0096 unless account says otherwise
          ngc.sync = false;
        }
      '';
      description = ''
        Written to ~/.config/aicentre/config.toml. Tables dtu, gefion and ngc
        take user, alias, account and sync; see https://hpc.aicentre.dk/cli/.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile."aicentre/config.toml" = lib.mkIf (cfg.settings != { }) {
      source = toml.generate "aicentre-config.toml" cfg.settings;
    };
  };
}
