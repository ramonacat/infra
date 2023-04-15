variable "hcloud_token" {}

terraform {
    backend "remote" {
        organization = "ramona"

        workspaces {
            name = "infra"
        }
    }
}

terraform {
  required_providers {
    hcloud = {
      source = "hetznercloud/hcloud"
    }
  }
  required_version = ">= 0.13"
}

provider "hcloud" {
  token = "${var.hcloud_token}"
}

resource "hcloud_ssh_key" "ramona" {
    name = "Ramona Nitrokey"
    public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQCatH7XWmY6oZPSe3woP2swvJ4/stZrpaVWNg6FMcs87xEtCr/sIkj/rm41gD6F3k3Z6jhxqBKgZcr45aW07xlB//KfYs3kb0PYDsn3KrwCPBjHwRypuPvyagCUDAbD9wnhpEr9iHEbhW2yNEDC5E1c3ak/fNjewCZMpqo645gQ6siFAnwEnqTQR0lF3B/hdmAA/j+efQ3ghjiI6+O3uQ0o5coCNa4tCrq3yqsyA7eI0jhT1Ij8SE54ren3dwndq1JoGNg7DCtozl3fCgHVUrdWeW2kcB1A/Ta+jcmcB10Rv9ZevU2wYvZIEYXG1hSjM8Zrr7JwAcXkG/mb3lGnYnU49YxNqT4vwD0ZyY8d5M9Hvw065+y7Y45+/ScevmIGn/fn/9TbZHdPdSKM1UFMICUctT6VH6ShhEkbiQ38E3GnA1n3mnsOnxaBT5hVJxr13yLV8ULU/8not6SMU/3xP2rZj6JP7xtHJP/29Nd4N7gm6adz3wbS1aRJosVr3ZbA1qTaB/m4EBRTfNYtifUbdQkFbrnlNmVNb5ixhS1ZLZq4aRPmp6MH034sQ9HZSrtMMSO5B9TXHCb3zxexR6BBtIjZHBqwuu3krMWh9kOW3wNFWmEWdy5vLUcVVoXSaGqICQwG/HOKGNdzGumFDnPfvayVVCxu67s2b82oTtkbd+mjMQ== openpgp:0xCF7158EB"
}

resource "hcloud_network" "mainnet" {
    name = "mainnet"
    ip_range = "10.69.100.0/24"
}

resource "hcloud_network_route" "world" {
    network_id = hcloud_network.mainnet.id
    destination = "0.0.0.0/0"
    gateway = "10.69.100.1"
}

resource "hcloud_network_subnet" "mainnet-subnet0" {
    network_id = hcloud_network.mainnet.id
    type = "cloud"
    ip_range = "10.69.100.0/24"
    network_zone = "eu-central"
}

resource "hcloud_server" "jump" {
    name = "jump"
    location = "fsn1"
    image = "ubuntu-22.04"
    server_type = "cax11"
    ssh_keys = [ hcloud_ssh_key.ramona.id ]
    user_data = templatefile("jump.cloud-config.yaml", { ip = "10.69.100.5" })

    network {
        network_id = hcloud_network.mainnet.id
        ip = "10.69.100.5"
    }

    depends_on = [
      hcloud_network_subnet.mainnet-subnet0
    ]
}

locals {
    k8s-nodes = {
        node0 = { ip = "10.69.100.20" },
        node1 = { ip = "10.69.100.21" },
        node2 = { ip = "10.69.100.22" },
    }
}

resource "hcloud_server" "nodes" {
    for_each = local.k8s-nodes
    name = each.key
    location = "fsn1"
    image = "ubuntu-22.04"
    server_type = "cax11"
    ssh_keys = [ hcloud_ssh_key.ramona.id ]
    user_data = templatefile("nodes.cloud-config.yaml", { ip = each.value.ip })

    public_net {
      ipv4_enabled = false
      ipv6_enabled = false
    }

    network {
        network_id = hcloud_network.mainnet.id
        ip = each.value.ip
    }

    depends_on = [
      hcloud_network_subnet.mainnet-subnet0
    ]
}

output "jump_ip" {
  value = hcloud_server.jump.ipv4_address
}