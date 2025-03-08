{ config, pkgs, ... }:

{
  imports =
    [ # Include the results of the hardware scan
      ./hardware-configuration.nix
    ];

  # Use the systemd-boot EFI boot loader
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # Networking
  networking.hostName = "business-directory";
  networking.networkmanager.enable = true;
  
  # Enable Docker
  virtualisation.docker = {
    enable = true;
    autoPrune.enable = true;
  };

  # Enable Docker Compose
  environment.systemPackages = with pkgs; [
    docker-compose
    git
    vim
    htop
    curl
    wget
  ];

  # Firewall settings
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ 22 80 443 8000 5000 ];
  };

  # Enable the OpenSSH daemon
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      PermitRootLogin = "no";
    };
  };

  # User configuration
  users.users.deploy = {
    isNormalUser = true;
    extraGroups = [ "wheel" "docker" "networkmanager" ];
    openssh.authorizedKeys.keys = [
      # Add your SSH public key here
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQD..."
    ];
  };

  # Set your time zone
  time.timeZone = "UTC";

  # System-wide environment variables
  environment.variables = {
    EDITOR = "vim";
  };

  # This value determines the NixOS release with which your system is to be compatible
  system.stateVersion = "24.05";
} 