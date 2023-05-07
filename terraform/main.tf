variable "hcloud_token" {}
variable "ovh_application_key" {}
variable "ovh_application_secret" {}
variable "ovh_consumer_key" {}
variable "vultr_api_key" {}
variable "github_token" {}

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
    ovh = {
      source = "ovh/ovh"
      version = "0.29.0"
    }
    vultr = {
      source = "vultr/vultr"
      version = "2.15.0"
    }
    flux = {
      source = "fluxcd/flux"
    }
    github = {
      source  = "integrations/github"
      version = ">=5.18.0"
    }
  }
  required_version = ">= 0.13"
}

provider "ovh" {
  endpoint           = "ovh-eu"
  application_key    = "${var.ovh_application_key}"
  application_secret = "${var.ovh_application_secret}"
  consumer_key       = "${var.ovh_consumer_key}"
}

provider "hcloud" {
  token = "${var.hcloud_token}"
}

provider "vultr" {
  api_key = "${var.vultr_api_key}"
}

provider "flux" {
  kubernetes = {
    host = vultr_kubernetes.k8s.endpoint
    client_certificate = vultr_kubernetes.k8s.client_certificate
    client_key = vultr_kubernetes.k8s.client_key
    cluster_ca_certificate = vultr_kubernetes.k8s.cluster_ca_certificate
  }
  git = {
    url  = "ssh://git@github.com/Agares/infra.git"
    ssh = {
      username    = "git"
      private_key = tls_private_key.flux.private_key_pem
    }
  }
}

provider "github" {
  owner = "Agares"
  token = "${var.github_token}"
}

resource "tls_private_key" "flux" {
  algorithm   = "ECDSA"
  ecdsa_curve = "P256"
}

resource "github_repository_deploy_key" "this" {
  title      = "Flux"
  repository = "infra"
  key        = tls_private_key.flux.public_key_openssh
  read_only  = "false"
}

resource "vultr_kubernetes" "k8s" {
  region="ewr"
  label="ramona-infra"
  version="v1.26.2+2"

  node_pools {
    node_quantity = 2
    plan = "vc2-1c-2gb"
    label="main-nodes"
    auto_scaler = true
    min_nodes = 2
    max_nodes = 3
  }
}

resource "flux_bootstrap_git" "this" {
  depends_on = [github_repository_deploy_key.this]

  path = "clusters/ramona"
}