{ self, ... }:

{
  flake.nixosModules.default =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      cfg = config.services.telegram-fwd;

      toml = pkgs.formats.toml { };

      configFile =
        if cfg.config == null then
          null
        else if lib.isPath cfg.config || lib.isString cfg.config then
          cfg.config
        else
          toml.generate "config.toml" cfg.config;
    in
    {
      options.services.telegram-fwd = {
        enable = lib.mkEnableOption "enable telegram-fwd service";

        package = lib.mkOption {
          type = lib.types.package;
          default = self.packages.${pkgs.system}.default;
          description = "telegram-fwd package to use";
        };

        config = lib.mkOption {
          type =
            with lib.types;
            oneOf [
              str
              path
              toml.type
              null
            ];
          description = "configuration settings for telegram-fwd. Also accepts paths (string or path type) to a config file.";
        };
      };

      config = lib.mkIf cfg.enable {
        systemd.services.telegram-fwd = {
          description = "Telegram Forwarder Service";
          wantedBy = [ "multi-user.target" ];
          after = [ "network.target" ];

          serviceConfig = {
            Type = "simple";
            ExecStart = "${cfg.package}/bin/telegram-fwd --config ${lib.escapeShellArg configFile}";
            Restart = "on-failure";
            RestartSec = "5s";

            NoNewPrivileges = true;
            PrivateTmp = true;
            ProtectSystem = "strict";
            ProtectHome = true;

            StateDirectory = "telegram-fwd";
            WorkingDirectory = "%S";
          };
        };
      };
    };

}
