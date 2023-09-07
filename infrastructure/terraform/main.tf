variable "hcloud_token" {}
variable "ovh_application_key" {}
variable "ovh_application_secret" {}
variable "ovh_consumer_key" {}
variable "vultr_api_key" {}
variable "github_token" {}
variable "google_credentials" {}
variable "honeycomb_key" {}

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
      source  = "hashicorp/kubernetes"
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
    host                   = format("https://%s:6443", vultr_kubernetes.k8s.ip)
    client_certificate     = base64decode(vultr_kubernetes.k8s.client_certificate)
    client_key             = base64decode(vultr_kubernetes.k8s.client_key)
    cluster_ca_certificate = base64decode(vultr_kubernetes.k8s.cluster_ca_certificate)
  }
  git = {
    url = "ssh://git@github.com/ramonacat/infra.git"
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
  host                   = format("https://%s:6443", vultr_kubernetes.k8s.ip)
  client_key             = base64decode(vultr_kubernetes.k8s.client_key)
  client_certificate     = base64decode(vultr_kubernetes.k8s.client_certificate)
  cluster_ca_certificate = base64decode(vultr_kubernetes.k8s.cluster_ca_certificate)
}

provider "vultr" {
  api_key = var.vultr_api_key
}

provider "google" {
  project     = "ramona-infra"
  region      = "europe-west3"
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

  path = "infrastructure/clusters/ramona"
}

resource "kubernetes_secret" "ovh_credentials" {
  metadata {
    name      = "ovh-credentials"
    namespace = "default"
  }
  data = {
    OVH_APPLICATION_KEY    = var.ovh_application_key
    OVH_APPLICATION_SECRET = var.ovh_application_secret
    OVH_CONSUMER_KEY       = var.ovh_consumer_key
  }
}

resource "kubernetes_secret" "google_credentials" {
  metadata {
    name      = "google-credentials"
    namespace = "default"
  }
  data = {
    GOOGLE_CREDENTIALS = var.google_credentials
  }
}

resource "kubernetes_secret" "honeycomb_key" {
  metadata {
    name      = "honeycomb-key"
    namespace = "default"
  }
  data = {
    HONEYCOMB_KEY = var.honeycomb_key
  }
}

resource "vultr_kubernetes" "k8s" {
  region  = "ewr"
  label   = "ramona-infra"
  version = "v1.27.2+1"

  node_pools {
    node_quantity = 2
    plan          = "vc2-1c-2gb"
    label         = "main-nodes"
    auto_scaler   = true
    min_nodes     = 2
    max_nodes     = 3
  }
}

resource "google_project_service" "dns" {
    project = "ramona-infra"
    service = "dns.googleapis.com"

    depends_on = [ google_project_service.cloudresourcemanager ]
}

resource "google_project_service" "cloudresourcemanager" {
    project = "ramona-infra"
    service = "cloudresourcemanager.googleapis.com"
}

resource "google_dns_managed_zone" "ramona-fun" {
  name = "ramona-fun"
  dns_name = "ramona.fun."

  depends_on = [ google_project_service.dns ]
}