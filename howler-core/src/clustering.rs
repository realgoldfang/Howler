use crate::models::Sighting;
use anyhow::Result;
use geo::{algorithm::convex_hull::ConvexHull, point, Coord};
use geojson::{Geometry, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DBSCAN clustering parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbscanParams {
    /// Maximum distance between points to be considered in the same neighborhood (in kilometers)
    pub epsilon: f64,
    /// Minimum number of points to form a cluster
    pub min_points: usize,
}

impl Default for DbscanParams {
    fn default() -> Self {
        Self {
            epsilon: 5.0, // 5 km
            min_points: 5,
        }
    }
}

/// Detected pack territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackTerritory {
    /// Unique territory ID
    pub id: usize,
    /// List of sighting IDs in this territory
    pub sighting_ids: Vec<i64>,
    /// Territory boundary as GeoJSON polygon
    pub boundary: Option<String>,
    /// Estimated territory size in square kilometers
    pub area_km2: Option<f64>,
    /// Center point (latitude, longitude)
    pub center: (f64, f64),
    /// Number of sightings in territory
    pub sighting_count: usize,
}

/// DBSCAN clustering result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringResult {
    /// Detected territories
    pub territories: Vec<PackTerritory>,
    /// Number of noise points (sightings not in any territory)
    pub noise_count: usize,
    /// Total number of sightings processed
    pub total_sightings: usize,
}

/// DBSCAN clustering algorithm implementation
pub fn dbscan_cluster(sightings: &[Sighting], params: &DbscanParams) -> Result<ClusteringResult> {
    if sightings.is_empty() {
        return Ok(ClusteringResult {
            territories: vec![],
            noise_count: 0,
            total_sightings: 0,
        });
    }

    let mut visited = vec![false; sightings.len()];
    let mut cluster_labels: Vec<Option<usize>> = vec![None; sightings.len()];
    let mut cluster_id = 0;

    for i in 0..sightings.len() {
        if visited[i] {
            continue;
        }
        visited[i] = true;

        let neighbors = region_query(sightings, i, params);

        if neighbors.len() < params.min_points {
            // Mark as noise
            cluster_labels[i] = None;
        } else {
            // Start new cluster
            cluster_labels[i] = Some(cluster_id);
            expand_cluster(
                sightings,
                &mut visited,
                &mut cluster_labels,
                &mut neighbors.clone(),
                cluster_id,
                params,
            );
            cluster_id += 1;
        }
    }

    // Build territories from clusters
    let mut cluster_map: HashMap<usize, Vec<i64>> = HashMap::new();
    for (i, label) in cluster_labels.iter().enumerate() {
        if let Some(cid) = label {
            if let Some(id) = sightings[i].id {
                cluster_map.entry(*cid).or_default().push(id);
            }
        }
    }

    let mut territories = Vec::new();
    for (cid, sighting_ids) in cluster_map {
        let cluster_sightings: Vec<&Sighting> = sighting_ids
            .iter()
            .filter_map(|&id| sightings.iter().find(|s| s.id == Some(id)))
            .collect();

        let center = calculate_center(&cluster_sightings);
        let boundary = calculate_convex_hull(&cluster_sightings);
        let area_km2 = calculate_area(&boundary);

        territories.push(PackTerritory {
            id: cid,
            sighting_ids,
            boundary,
            area_km2,
            center,
            sighting_count: cluster_sightings.len(),
        });
    }

    let noise_count = cluster_labels.iter().filter(|l| l.is_none()).count();

    Ok(ClusteringResult {
        territories,
        noise_count,
        total_sightings: sightings.len(),
    })
}

/// Find all points within epsilon distance of point at index
fn region_query(sightings: &[Sighting], index: usize, params: &DbscanParams) -> Vec<usize> {
    let mut neighbors = Vec::new();
    let _p1 = point!(x: sightings[index].longitude, y: sightings[index].latitude);

    for (i, sighting) in sightings.iter().enumerate() {
        let _p2 = point!(x: sighting.longitude, y: sighting.latitude);
        let distance_km = haversine_distance(
            sightings[index].latitude,
            sightings[index].longitude,
            sighting.latitude,
            sighting.longitude,
        );

        if distance_km <= params.epsilon {
            neighbors.push(i);
        }
    }

    neighbors
}

/// Expand cluster by adding density-reachable points
fn expand_cluster(
    sightings: &[Sighting],
    visited: &mut [bool],
    cluster_labels: &mut [Option<usize>],
    neighbors: &mut Vec<usize>,
    cluster_id: usize,
    params: &DbscanParams,
) {
    let mut i = 0;
    while i < neighbors.len() {
        let neighbor_idx = neighbors[i];

        if !visited[neighbor_idx] {
            visited[neighbor_idx] = true;
            let new_neighbors = region_query(sightings, neighbor_idx, params);

            if new_neighbors.len() >= params.min_points {
                for &nn in &new_neighbors {
                    if !neighbors.contains(&nn) {
                        neighbors.push(nn);
                    }
                }
            }
        }

        if cluster_labels[neighbor_idx].is_none() {
            cluster_labels[neighbor_idx] = Some(cluster_id);
        }

        i += 1;
    }
}

/// Calculate center point of a cluster
fn calculate_center(sightings: &[&Sighting]) -> (f64, f64) {
    if sightings.is_empty() {
        return (0.0, 0.0);
    }

    let sum_lat: f64 = sightings.iter().map(|s| s.latitude).sum();
    let sum_lon: f64 = sightings.iter().map(|s| s.longitude).sum();
    let count = sightings.len() as f64;

    (sum_lat / count, sum_lon / count)
}

/// Calculate convex hull boundary as GeoJSON polygon
fn calculate_convex_hull(sightings: &[&Sighting]) -> Option<String> {
    if sightings.len() < 3 {
        return None;
    }

    let points: Vec<Coord<f64>> = sightings
        .iter()
        .map(|s| Coord {
            x: s.longitude,
            y: s.latitude,
        })
        .collect();

    let polygon = geo::Polygon::<f64>::new(points.into(), vec![]);
    let hull = polygon.convex_hull();

    let exterior = hull.exterior();
    let coords: Vec<Vec<f64>> = exterior.points().map(|p| vec![p.x(), p.y()]).collect();

    let geometry = Geometry::new(Value::Polygon(vec![coords]));
    Some(serde_json::to_string(&geometry).unwrap_or_default())
}

/// Calculate area of polygon in square kilometers
fn calculate_area(boundary: &Option<String>) -> Option<f64> {
    let boundary_str = boundary.as_ref()?;
    let geometry: Geometry = serde_json::from_str(boundary_str).ok()?;

    match geometry.value {
        Value::Polygon(ref rings) => {
            if let Some(exterior) = rings.first() {
                if exterior.len() < 4 {
                    return None;
                }

                // Use shoelace formula for area calculation
                let mut area = 0.0;
                for i in 0..exterior.len() - 1 {
                    let (x1, y1) = (exterior[i][0], exterior[i][1]);
                    let (x2, y2) = (exterior[i + 1][0], exterior[i + 1][1]);
                    area += (x1 * y2) - (x2 * y1);
                }
                area = area.abs() / 2.0;

                // Convert from degrees^2 to approximate km^2
                // This is a rough approximation; for precise calculations, use proper projection
                Some(area * 111.32 * 111.32)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Calculate Haversine distance between two points in kilometers
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a = (dlat / 2.0).sin() * (dlat / 2.0).sin()
        + (dlon / 2.0).sin() * (dlon / 2.0).sin() * lat1_rad.cos() * lat2_rad.cos();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Detect territory overlaps
pub fn detect_overlaps(
    territories: &[PackTerritory],
    threshold_km: f64,
) -> Vec<(usize, usize, f64)> {
    let mut overlaps = Vec::new();

    for i in 0..territories.len() {
        for j in (i + 1)..territories.len() {
            let distance = haversine_distance(
                territories[i].center.0,
                territories[i].center.1,
                territories[j].center.0,
                territories[j].center.1,
            );

            if distance < threshold_km {
                overlaps.push((i, j, distance));
            }
        }
    }

    overlaps
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_sighting(lat: f64, lon: f64, id: i64) -> Sighting {
        Sighting {
            id: Some(id),
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: lat,
            longitude: lon,
            observed_on: Utc::now(),
            source: crate::models::Source::GBIF,
            source_id: format!("test_{}", id),
            details: None,
        }
    }

    #[test]
    fn test_haversine_distance() {
        let distance = haversine_distance(0.0, 0.0, 0.0, 1.0);
        assert!(distance > 100.0 && distance < 120.0); // ~111 km per degree
    }

    #[test]
    fn test_dbscan_empty() {
        let sightings = vec![];
        let params = DbscanParams::default();
        let result = dbscan_cluster(&sightings, &params).unwrap();
        assert_eq!(result.total_sightings, 0);
        assert!(result.territories.is_empty());
    }

    #[test]
    fn test_dbscan_single_cluster() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1),
            create_test_sighting(45.01, -122.01, 2),
            create_test_sighting(45.02, -122.02, 3),
            create_test_sighting(45.03, -122.03, 4),
            create_test_sighting(45.04, -122.04, 5),
        ];

        let params = DbscanParams {
            epsilon: 10.0,
            min_points: 3,
        };

        let result = dbscan_cluster(&sightings, &params).unwrap();
        assert_eq!(result.territories.len(), 1);
        assert_eq!(result.territories[0].sighting_count, 5);
    }

    #[test]
    fn test_dbscan_noise() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1),
            create_test_sighting(45.01, -122.01, 2),
            create_test_sighting(50.0, -120.0, 3), // Far away - should be noise
        ];

        let params = DbscanParams {
            epsilon: 5.0,
            min_points: 2,
        };

        let result = dbscan_cluster(&sightings, &params).unwrap();
        assert_eq!(result.territories.len(), 1);
        assert_eq!(result.noise_count, 1);
    }

    #[test]
    fn test_calculate_center() {
        let sightings = [
            create_test_sighting(45.0, -122.0, 1),
            create_test_sighting(47.0, -124.0, 2),
        ];

        let center = calculate_center(&sightings.iter().collect::<Vec<_>>());
        assert!((center.0 - 46.0).abs() < 0.01);
        assert!((center.1 - (-123.0)).abs() < 0.01);
    }
}
