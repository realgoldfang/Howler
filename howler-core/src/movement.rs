use crate::models::Sighting;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Movement between two sightings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    /// From sighting ID
    pub from_id: i64,
    /// To sighting ID
    pub to_id: i64,
    /// Distance traveled in kilometers
    pub distance_km: f64,
    /// Bearing in degrees (0-360, where 0 is North)
    pub bearing_degrees: f64,
    /// Duration in seconds
    pub duration_seconds: i64,
    /// Speed in km/h
    pub speed_kmh: f64,
}

/// Movement pattern type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MovementPattern {
    /// Random movement
    Random,
    /// Circular/territorial movement
    Circular,
    /// Linear movement (migration or dispersal)
    Linear,
    /// Stationary (little movement)
    Stationary,
}

/// Movement analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementAnalysis {
    /// All calculated movements
    pub movements: Vec<Movement>,
    /// Average speed in km/h
    pub average_speed_kmh: f64,
    /// Maximum speed in km/h
    pub max_speed_kmh: f64,
    /// Total distance traveled in kilometers
    pub total_distance_km: f64,
    /// Detected movement pattern
    pub pattern: MovementPattern,
    /// Average bearing in degrees
    pub average_bearing_degrees: f64,
}

/// Calculate bearing between two points
pub fn calculate_bearing(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let y = dlon.sin() * lat2_rad.cos();
    let x = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * dlon.cos();

    let bearing = y.atan2(x).to_degrees();
    (bearing + 360.0) % 360.0
}

/// Calculate Haversine distance between two points in kilometers
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
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

/// Analyze movements between sightings
pub fn analyze_movements(sightings: &[Sighting]) -> Result<MovementAnalysis> {
    if sightings.len() < 2 {
        return Ok(MovementAnalysis {
            movements: vec![],
            average_speed_kmh: 0.0,
            max_speed_kmh: 0.0,
            total_distance_km: 0.0,
            pattern: MovementPattern::Stationary,
            average_bearing_degrees: 0.0,
        });
    }

    // Sort sightings by time
    let mut sorted_sightings = sightings.to_vec();
    sorted_sightings.sort_by_key(|a| a.observed_on);

    let mut movements = Vec::new();
    let mut total_distance = 0.0;
    let mut total_speed = 0.0;
    let mut max_speed: f64 = 0.0;
    let mut total_bearing = 0.0;

    for window in sorted_sightings.windows(2) {
        let from = &window[0];
        let to = &window[1];

        if let (Some(from_id), Some(to_id)) = (from.id, to.id) {
            let distance =
                haversine_distance(from.latitude, from.longitude, to.latitude, to.longitude);
            let bearing =
                calculate_bearing(from.latitude, from.longitude, to.latitude, to.longitude);
            let duration = to.observed_on.signed_duration_since(from.observed_on);
            let duration_seconds = duration.num_seconds();

            let speed_kmh = if duration_seconds > 0 {
                (distance / duration_seconds as f64) * 3600.0
            } else {
                0.0
            };

            total_distance += distance;
            total_speed += speed_kmh;
            max_speed = max_speed.max(speed_kmh);
            total_bearing += bearing;

            movements.push(Movement {
                from_id,
                to_id,
                distance_km: distance,
                bearing_degrees: bearing,
                duration_seconds,
                speed_kmh,
            });
        }
    }

    let average_speed = if !movements.is_empty() {
        total_speed / movements.len() as f64
    } else {
        0.0
    };

    let average_bearing = if !movements.is_empty() {
        total_bearing / movements.len() as f64
    } else {
        0.0
    };

    let pattern = detect_movement_pattern(&movements);

    Ok(MovementAnalysis {
        movements,
        average_speed_kmh: average_speed,
        max_speed_kmh: max_speed,
        total_distance_km: total_distance,
        pattern,
        average_bearing_degrees: average_bearing,
    })
}

/// Detect movement pattern from movements
fn detect_movement_pattern(movements: &[Movement]) -> MovementPattern {
    if movements.is_empty() {
        return MovementPattern::Stationary;
    }

    let total_distance: f64 = movements.iter().map(|m| m.distance_km).sum();
    let avg_distance = total_distance / movements.len() as f64;

    // Calculate bearing variance
    let bearings: Vec<f64> = movements.iter().map(|m| m.bearing_degrees).collect();
    let avg_bearing = bearings.iter().sum::<f64>() / bearings.len() as f64;
    let bearing_variance: f64 = bearings
        .iter()
        .map(|&b| {
            let diff = (b - avg_bearing).to_radians();
            diff.sin().powi(2) + diff.cos().powi(2)
        })
        .sum::<f64>()
        / bearings.len() as f64;

    // Determine pattern based on statistics
    if avg_distance < 0.5 {
        MovementPattern::Stationary
    } else if bearing_variance < 0.5 {
        MovementPattern::Linear
    } else if bearing_variance > 1.5 {
        MovementPattern::Circular
    } else {
        MovementPattern::Random
    }
}

/// Calculate home range using minimum convex polygon
pub fn calculate_home_range(sightings: &[Sighting]) -> Option<Vec<(f64, f64)>> {
    if sightings.len() < 3 {
        return None;
    }

    let points: Vec<(f64, f64)> = sightings
        .iter()
        .map(|s| (s.longitude, s.latitude))
        .collect();

    // Use convex hull to calculate home range
    use geo::{algorithm::convex_hull::ConvexHull, Point};
    let geo_points: Vec<Point<f64>> = points
        .iter()
        .map(|&(lon, lat)| Point::new(lon, lat))
        .collect();

    let line_string = geo::LineString::from(geo_points);
    let polygon = geo::Polygon::new(line_string, vec![]);
    let hull = polygon.convex_hull();

    let exterior = hull.exterior();
    Some(exterior.points().map(|p| (p.y(), p.x())).collect())
}

/// Calculate daily movement statistics
pub fn calculate_daily_statistics(sightings: &[Sighting]) -> Vec<(String, f64, f64)> {
    let mut daily_stats: std::collections::HashMap<String, (f64, usize)> =
        std::collections::HashMap::new();

    for window in sightings.windows(2) {
        let from = &window[0];
        let to = &window[1];

        let date = from.observed_on.format("%Y-%m-%d").to_string();
        let distance = haversine_distance(from.latitude, from.longitude, to.latitude, to.longitude);

        let entry = daily_stats.entry(date).or_insert((0.0, 0));
        entry.0 += distance;
        entry.1 += 1;
    }

    let mut result: Vec<(String, f64, f64)> = daily_stats
        .into_iter()
        .map(|(date, (total_distance, count))| {
            let avg_distance = if count > 0 {
                total_distance / count as f64
            } else {
                0.0
            };
            (date, total_distance, avg_distance)
        })
        .collect();

    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Source;
    use chrono::{Duration, Utc};

    fn create_test_sighting(lat: f64, lon: f64, id: i64, hours_ago: i64) -> Sighting {
        Sighting {
            id: Some(id),
            species: "Canis lupus".to_string(),
            scientific_name: Some("Canis lupus".to_string()),
            latitude: lat,
            longitude: lon,
            observed_on: Utc::now() - Duration::hours(hours_ago),
            source: Source::GBIF,
            source_id: format!("test_{}", id),
            details: None,
        }
    }

    #[test]
    fn test_haversine_distance() {
        let distance = haversine_distance(0.0, 0.0, 0.0, 1.0);
        assert!(distance > 100.0 && distance < 120.0);
    }

    #[test]
    fn test_calculate_bearing() {
        // North
        let bearing = calculate_bearing(0.0, 0.0, 1.0, 0.0);
        assert!((bearing - 0.0).abs() < 1.0);

        // East
        let bearing = calculate_bearing(0.0, 0.0, 0.0, 1.0);
        assert!((bearing - 90.0).abs() < 1.0);

        // South
        let bearing = calculate_bearing(0.0, 0.0, -1.0, 0.0);
        assert!((bearing - 180.0).abs() < 1.0);

        // West
        let bearing = calculate_bearing(0.0, 0.0, 0.0, -1.0);
        assert!((bearing - 270.0).abs() < 1.0);
    }

    #[test]
    fn test_analyze_movements_empty() {
        let sightings = vec![];
        let result = analyze_movements(&sightings).unwrap();
        assert_eq!(result.movements.len(), 0);
        assert_eq!(result.pattern, MovementPattern::Stationary);
    }

    #[test]
    fn test_analyze_movements_single() {
        let sightings = vec![create_test_sighting(45.0, -122.0, 1, 0)];
        let result = analyze_movements(&sightings).unwrap();
        assert_eq!(result.movements.len(), 0);
    }

    #[test]
    fn test_analyze_movements_linear() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 4),
            create_test_sighting(45.1, -122.0, 2, 3),
            create_test_sighting(45.2, -122.0, 3, 2),
            create_test_sighting(45.3, -122.0, 4, 1),
            create_test_sighting(45.4, -122.0, 5, 0),
        ];

        let result = analyze_movements(&sightings).unwrap();
        assert_eq!(result.movements.len(), 4);
        assert!(result.total_distance_km > 0.0);
    }

    #[test]
    fn test_calculate_daily_statistics() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 25),
            create_test_sighting(45.1, -122.0, 2, 24),
            create_test_sighting(45.2, -122.0, 3, 1),
            create_test_sighting(45.3, -122.0, 4, 0),
        ];

        let stats = calculate_daily_statistics(&sightings);
        assert!(!stats.is_empty());
    }
}
