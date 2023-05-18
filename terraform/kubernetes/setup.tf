variable "vultr_api_key" {}

terraform {
  required_providers {
    vultr = {
      source  = "vultr/vultr"
      version = "2.15.0"
    }
  }
  required_version = ">= 0.13"
}

provider "vultr" {
  api_key = var.vultr_api_key
}

resource "vultr_kubernetes" "k8s" {
  region  = "ewr"
  label   = "ramona-infra"
  version = "v1.26.2+2"

  node_pools {
    node_quantity = 2
    plan          = "vc2-1c-2gb"
    label         = "main-nodes"
    auto_scaler   = true
    min_nodes     = 2
    max_nodes     = 3
  }
}

output "cluster_endpoint" {
    value = vultr_kubernetes.k8s.endpoint
}

output "client_certificate" {
    value = vultr_kubernetes.k8s.client_certificate
}
output "client_key" {
    value = vultr_kubernetes.k8s.client_key
}
output "cluster_ca_certificate" {    
    value = vultr_kubernetes.k8s.cluster_ca_certificate
}