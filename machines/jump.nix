{ lib, modulesPath, pkgs, ... }:
{
  imports = [
    (modulesPath + "/profiles/qemu-guest.nix")
  ];

  config = {
    boot.loader.grub = {
      efiSupport = true;
      efiInstallAsRemovable = true;
      device = "nodev";
    };
    fileSystems."/boot" = { device = "/dev/sda15"; fsType = "vfat"; };
    boot.initrd.availableKernelModules = [ "ata_piix" "uhci_hcd" "xen_blkfront" ];
    boot.initrd.kernelModules = [ "nvme" ];
    fileSystems."/" = { device = "/dev/sda1"; fsType = "ext4"; };
    console.keyMap = "pl";
    nixpkgs.config.allowUnfree = true;
    nix.settings.experimental-features = [ "nix-command flakes" ];

    boot.tmp.cleanOnBoot = true;
    zramSwap.enable = true;
    networking.hostName = "jump";
    networking.domain = "";
    services.openssh.enable = true;
    users.users.root.openssh.authorizedKeys.keys = [
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQCatH7XWmY6oZPSe3woP2swvJ4/stZrpaVWNg6FMcs87xEtCr/sIkj/rm41gD6F3k3Z6jhxqBKgZcr45aW07xlB//KfYs3kb0PYDsn3KrwCPBjHwRypuPvyagCUDAbD9wnhpEr9iHEbhW2yNEDC5E1c3ak/fNjewCZMpqo645gQ6siFAnwEnqTQR0lF3B/hdmAA/j+efQ3ghjiI6+O3uQ0o5coCNa4tCrq3yqsyA7eI0jhT1Ij8SE54ren3dwndq1JoGNg7DCtozl3fCgHVUrdWeW2kcB1A/Ta+jcmcB10Rv9ZevU2wYvZIEYXG1hSjM8Zrr7JwAcXkG/mb3lGnYnU49YxNqT4vwD0ZyY8d5M9Hvw065+y7Y45+/ScevmIGn/fn/9TbZHdPdSKM1UFMICUctT6VH6ShhEkbiQ38E3GnA1n3mnsOnxaBT5hVJxr13yLV8ULU/8not6SMU/3xP2rZj6JP7xtHJP/29Nd4N7gm6adz3wbS1aRJosVr3ZbA1qTaB/m4EBRTfNYtifUbdQkFbrnlNmVNb5ixhS1ZLZq4aRPmp6MH034sQ9HZSrtMMSO5B9TXHCb3zxexR6BBtIjZHBqwuu3krMWh9kOW3wNFWmEWdy5vLUcVVoXSaGqICQwG/HOKGNdzGumFDnPfvayVVCxu67s2b82oTtkbd+mjMQ== openpgp:0xCF7158EB"
    ];

    networking = {
      nameservers = [ "8.8.8.8" ];
      defaultGateway = "172.31.1.1";
      defaultGateway6 = {
        address = "fe80::1";
        interface = "eth0";
      };
      usePredictableInterfaceNames = lib.mkForce false;
      interfaces = {
        eth0 = {
          ipv4.addresses = [
            { address = "78.46.191.39"; prefixLength = 32; }
          ];
          ipv6.addresses = [
            { address = "2a01:4f8:c012:858e::1"; prefixLength = 64; }
            { address = "fe80::9400:2ff:fe18:ce3d"; prefixLength = 64; }
          ];
          ipv4.routes = [{ address = "172.31.1.1"; prefixLength = 32; }];
          ipv6.routes = [{ address = "fe80::1"; prefixLength = 128; }];
        };
        enp7s0 = {
          useDHCP = true;
        };
      };
    };
    system.stateVersion = "23.05";
  };
}
