use log::debug;

/// Well-known GCP regions where Cloud Composer environments can be created.
/// This avoids a slow API call to the Compute Engine API at startup.
pub const GCP_REGIONS: &[&str] = &[
    // Africa
    "africa-south1",
    // Asia Pacific
    "asia-east1",
    "asia-east2",
    "asia-northeast1",
    "asia-northeast2",
    "asia-northeast3",
    "asia-south1",
    "asia-south2",
    "asia-southeast1",
    "asia-southeast2",
    // Australia
    "australia-southeast1",
    "australia-southeast2",
    // Europe
    "europe-central2",
    "europe-north1",
    "europe-southwest1",
    "europe-west1",
    "europe-west2",
    "europe-west3",
    "europe-west4",
    "europe-west6",
    "europe-west8",
    "europe-west9",
    "europe-west10",
    "europe-west12",
    // Middle East
    "me-central1",
    "me-central2",
    "me-west1",
    // North America
    "northamerica-northeast1",
    "northamerica-northeast2",
    "northamerica-south1",
    "us-central1",
    "us-east1",
    "us-east4",
    "us-east5",
    "us-south1",
    "us-west1",
    "us-west2",
    "us-west3",
    "us-west4",
    // South America
    "southamerica-east1",
    "southamerica-west1",
];

/// Attempts to get the default compute region from gcloud config.
/// Falls back gracefully if gcloud CLI is not installed.
pub fn get_gcloud_default_region() -> Option<String> {
    let output = std::process::Command::new("gcloud")
        .args(["config", "get", "compute/region"])
        .output()
        .ok()?;

    let region = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if region.is_empty() || region == "(unset)" {
        debug!("No default gcloud compute region set");
        None
    } else {
        debug!("Found gcloud default region: {region}");
        Some(region)
    }
}
