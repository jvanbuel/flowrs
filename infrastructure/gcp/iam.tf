# Service Account for Composer worker nodes
resource "google_service_account" "composer" {
  account_id   = "composer-worker"
  display_name = "Composer Worker Service Account"

  depends_on = [google_project_service.composer]
}

# Composer Worker role - covers GCS, logging, monitoring
resource "google_project_iam_member" "composer_worker" {
  project = var.project_id
  role    = "roles/composer.worker"
  member  = "serviceAccount:${google_service_account.composer.email}"
}
