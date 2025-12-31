# VPC Network for Composer
resource "google_compute_network" "composer" {
  name                    = "${var.environment_name}-network"
  auto_create_subnetworks = false

  depends_on = [google_project_service.compute]
}

# Subnetwork with secondary ranges for GKE (Composer 3 runs on GKE Autopilot)
resource "google_compute_subnetwork" "composer" {
  name          = "${var.environment_name}-subnet"
  ip_cidr_range = "10.0.0.0/24"
  region        = var.region
  network       = google_compute_network.composer.id

  secondary_ip_range {
    range_name    = "pods"
    ip_cidr_range = "10.1.0.0/16"
  }

  secondary_ip_range {
    range_name    = "services"
    ip_cidr_range = "10.2.0.0/20"
  }
}

# TODO: Add explicit firewall rules for production use.
# At minimum, allow ingress for Composer control plane, health checks,
# and SSH access; restrict egress as appropriate.
# See: https://cloud.google.com/composer/docs/composer-2/configure-private-ip#firewall_rules
