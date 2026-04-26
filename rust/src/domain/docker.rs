use anyhow::Result;
use bollard::container::ListContainersOptions;
use bollard::image::ListImagesOptions;
use bollard::Docker;
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

use super::traits::{ArtifactType, CleanupCategory, ScanResult, Scanner};

/// Scanner for Docker-related artifacts (dangling images, stopped containers, etc.)
pub struct DockerScanner;

impl Scanner for DockerScanner {
    fn scan(&self, _root: &Path) -> Result<Vec<ScanResult>> {
        let rt = Runtime::new()?;
        rt.block_on(self.scan_async())
    }
}

impl DockerScanner {
    async fn scan_async(&self) -> Result<Vec<ScanResult>> {
        let docker = match Docker::connect_with_local_defaults() {
            Ok(d) => d,
            Err(_) => return Ok(vec![]), // Docker might not be running
        };

        let mut results = Vec::new();

        // 1. Find dangling images
        let mut filters = std::collections::HashMap::new();
        filters.insert("dangling", vec!["true"]);

        if let Ok(images) = docker
            .list_images(Some(ListImagesOptions {
                filters,
                all: false,
                ..Default::default()
            }))
            .await
        {
            for image in images {
                if image.size > 0 {
                    results.push(ScanResult {
                        path: PathBuf::from(&image.id),
                        size_bytes: image.size as u64,
                        category: CleanupCategory::Cache,
                        artifact_type: ArtifactType::DockerImage(image.id),
                    });
                }
            }
        }

        // 2. Find exited containers
        let mut filters = std::collections::HashMap::new();
        filters.insert("status", vec!["exited"]);

        if let Ok(containers) = docker
            .list_containers(Some(ListContainersOptions {
                filters,
                all: true,
                ..Default::default()
            }))
            .await
        {
            for container in containers {
                // Bollard containers don't easily show size without an inspection
                // but we can at least identify them.
                if let Some(id) = container.id {
                    results.push(ScanResult {
                        path: PathBuf::from(&id),
                        size_bytes: 0, // Size is complex to calculate for containers
                        category: CleanupCategory::Temp,
                        artifact_type: ArtifactType::DockerContainer(id),
                    });
                }
            }
        }

        Ok(results)
    }
}
