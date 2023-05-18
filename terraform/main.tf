variable "hcloud_token" {}
variable "ovh_application_key" {}
variable "ovh_application_secret" {}
variable "ovh_consumer_key" {}
variable "vultr_api_key" {}
variable "github_token" {}

module "kubernetes" {
  source="./kubernetes/"
  vultr_api_key = var.vultr_api_key
}

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
      source  = "ovh/ovh"
      version = "0.29.0"
    }
    vultr = {
      source  = "vultr/vultr"
      version = "2.15.0"
    }
    flux = {
      source = "fluxcd/flux"
    }
    github = {
      source  = "integrations/github"
      version = ">=5.18.0"
    }
    kubernetes = {
      source = "hashicorp/kubernetes"
      version = "2.20.0"
    }
  }
  required_version = ">= 0.13"
}

provider "ovh" {
  endpoint           = "ovh-eu"
  application_key    = var.ovh_application_key
  application_secret = var.ovh_application_secret
  consumer_key       = var.ovh_consumer_key
}

provider "hcloud" {
  token = var.hcloud_token
}

provider "flux" {
  kubernetes = {
    # config_path            = module.kubernetes.kubectl_config
    host = module.kubernetes.cluster_endpoint
    client_certificate     = base64decode(module.kubernetes.client_certificate)
    client_key             = base64decode(module.kubernetes.client_key)
    cluster_ca_certificate = base64decode(module.kubernetes.cluster_ca_certificate)
  }
  git = {
    url = "ssh://git@github.com/Agares/infra.git"
    ssh = {
      username    = "git"
      private_key = tls_private_key.flux.private_key_pem
    }
  }
}

provider "github" {
  owner = "Agares"
  token = var.github_token
}

provider "kubernetes" {
  host = module.kubernetes.cluster_endpoint
  client_key             = base64decode(module.kubernetes.client_key)
  client_certificate     = base64decode(module.kubernetes.client_certificate)
  cluster_ca_certificate = base64decode(module.kubernetes.cluster_ca_certificate)
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

resource "flux_bootstrap_git" "this" {
  depends_on = [github_repository_deploy_key.this]

  path = "clusters/ramona"
}

resource "kubernetes_secret" "ovh_credentials" {
  metadata {
    name = "ovh-credentials"
    namespace = "default"
  }
  data = {
    OVH_APPLICATION_KEY = var.ovh_application_key
    OVH_APPLICATION_SECRET = var.ovh_application_secret
    OVH_CONSUMER_KEY = var.ovh_consumer_key
  }
}