use crate::models::Sighting;
use anyhow::Result;
use chrono::{Datelike, Timelike};
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use linfa_trees::DecisionTree;
use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Predicted next location for a wolf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationPrediction {
    /// Predicted latitude
    pub latitude: f64,
    /// Predicted longitude
    pub longitude: f64,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Prediction horizon in hours
    pub horizon_hours: u32,
}

/// Behavior classification result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BehaviorType {
    /// Wolf is stationary (denning, resting)
    Stationary,
    /// Wolf is patrolling territory
    Territorial,
    /// Wolf is moving linearly (dispersal/migration)
    Linear,
    /// Wolf is moving randomly
    Random,
    /// Wolf is returning to a central location
    CentralPlace,
}

/// Behavior prediction with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPrediction {
    /// Predicted behavior type
    pub behavior: BehaviorType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Features used for prediction
    pub features: BehaviorFeatures,
}

/// Features extracted from sighting data for ML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorFeatures {
    /// Average speed (km/h)
    pub avg_speed_kmh: f64,
    /// Speed variance
    pub speed_variance: f64,
    /// Turning angle variance (radians)
    pub turning_angle_variance: f64,
    /// Net displacement / total distance (straightness index)
    pub straightness_index: f64,
    /// Territory radius estimate (km)
    pub territory_radius_km: f64,
    /// Time since first sighting (hours)
    pub time_span_hours: f64,
    /// Number of sightings
    pub num_sightings: usize,
    /// Hour of day (0-23) - cyclical feature
    pub hour_of_day_sin: f64,
    pub hour_of_day_cos: f64,
    /// Day of year (1-365) - cyclical feature
    pub day_of_year_sin: f64,
    pub day_of_year_cos: f64,
}

/// Activity prediction for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPrediction {
    /// Hour of day (0-23)
    pub hour: u8,
    /// Predicted probability of activity (0.0 - 1.0)
    pub activity_probability: f64,
    /// Expected number of sightings
    pub expected_sightings: f64,
}

/// ML model for behavior prediction
pub struct BehaviorModel {
    /// Decision tree classifier for behavior type
    classifier: Option<DecisionTree<f64, usize>>,
    /// Linear regression for location prediction
    location_model: Option<linfa_linear::FittedLinearRegression<f64>>,
    /// Feature scaler
    scaler_mean: Option<Array1<f64>>,
    scaler_std: Option<Array1<f64>>,
}

impl Default for BehaviorModel {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorModel {
    pub fn new() -> Self {
        Self {
            classifier: None,
            location_model: None,
            scaler_mean: None,
            scaler_std: None,
        }
    }

    /// Extract features from a sequence of sightings
    pub fn extract_features(sightings: &[Sighting]) -> Result<BehaviorFeatures> {
        if sightings.len() < 2 {
            anyhow::bail!("Need at least 2 sightings for feature extraction");
        }

        let mut movements = Vec::new();
        let mut speeds = Vec::new();
        let mut bearings = Vec::new();

        // Sort by time
        let mut sorted = sightings.to_vec();
        sorted.sort_by_key(|s| s.observed_on);

        for window in sorted.windows(2) {
            let from = &window[0];
            let to = &window[1];

            let distance = crate::movement::haversine_distance(
                from.latitude,
                from.longitude,
                to.latitude,
                to.longitude,
            );
            let bearing = crate::movement::calculate_bearing(
                from.latitude,
                from.longitude,
                to.latitude,
                to.longitude,
            );
            let duration = to
                .observed_on
                .signed_duration_since(from.observed_on)
                .num_seconds() as f64
                / 3600.0;

            if duration > 0.0 {
                let speed = distance / duration;
                speeds.push(speed);
                movements.push((distance, bearing));
                bearings.push(bearing);
            }
        }

        if speeds.is_empty() {
            anyhow::bail!("No valid movements found");
        }

        // Calculate statistics
        let avg_speed = speeds.iter().sum::<f64>() / speeds.len() as f64;
        let speed_variance =
            speeds.iter().map(|s| (s - avg_speed).powi(2)).sum::<f64>() / speeds.len() as f64;

        // Turning angles
        let mut turning_angles = Vec::new();
        for i in 1..bearings.len() {
            let diff = (bearings[i] - bearings[i - 1]).abs();
            let angle = if diff > 180.0 { 360.0 - diff } else { diff };
            turning_angles.push(angle.to_radians());
        }
        let turning_angle_variance = if turning_angles.len() > 1 {
            let mean = turning_angles.iter().sum::<f64>() / turning_angles.len() as f64;
            turning_angles
                .iter()
                .map(|a| (a - mean).powi(2))
                .sum::<f64>()
                / turning_angles.len() as f64
        } else {
            0.0
        };

        // Straightness index (net displacement / total path length)
        let total_distance: f64 = movements.iter().map(|m| m.0).sum();
        let net_displacement = if sorted.len() >= 2 {
            let first = &sorted[0];
            let last = &sorted[sorted.len() - 1];
            crate::movement::haversine_distance(
                first.latitude,
                first.longitude,
                last.latitude,
                last.longitude,
            )
        } else {
            0.0
        };
        let straightness_index = if total_distance > 0.0 {
            net_displacement / total_distance
        } else {
            0.0
        };

        // Territory radius (max distance from centroid)
        let centroid_lat = sorted.iter().map(|s| s.latitude).sum::<f64>() / sorted.len() as f64;
        let centroid_lon = sorted.iter().map(|s| s.longitude).sum::<f64>() / sorted.len() as f64;
        let territory_radius_km = sorted
            .iter()
            .map(|s| {
                crate::movement::haversine_distance(
                    s.latitude,
                    s.longitude,
                    centroid_lat,
                    centroid_lon,
                )
            })
            .fold(0.0, f64::max);

        // Time span
        let first_time = sorted.first().unwrap().observed_on;
        let last_time = sorted.last().unwrap().observed_on;
        let time_span_hours =
            last_time.signed_duration_since(first_time).num_seconds() as f64 / 3600.0;

        // Temporal features (using last sighting)
        let last = sorted.last().unwrap();
        let hour = last.observed_on.hour() as f64;
        let day_of_year = last.observed_on.ordinal() as f64;

        Ok(BehaviorFeatures {
            avg_speed_kmh: avg_speed,
            speed_variance,
            turning_angle_variance,
            straightness_index,
            territory_radius_km,
            time_span_hours,
            num_sightings: sorted.len(),
            hour_of_day_sin: (hour * 2.0 * std::f64::consts::PI / 24.0).sin(),
            hour_of_day_cos: (hour * 2.0 * std::f64::consts::PI / 24.0).cos(),
            day_of_year_sin: (day_of_year * 2.0 * std::f64::consts::PI / 365.0).sin(),
            day_of_year_cos: (day_of_year * 2.0 * std::f64::consts::PI / 365.0).cos(),
        })
    }

    /// Convert features to array for ML
    fn features_to_array(features: &BehaviorFeatures) -> Array1<f64> {
        Array1::from(vec![
            features.avg_speed_kmh,
            features.speed_variance,
            features.turning_angle_variance,
            features.straightness_index,
            features.territory_radius_km,
            features.time_span_hours,
            features.num_sightings as f64,
            features.hour_of_day_sin,
            features.hour_of_day_cos,
            features.day_of_year_sin,
            features.day_of_year_cos,
        ])
    }

    /// Standardize features (zero mean, unit variance)
    fn standardize(&mut self, data: &mut Array2<f64>) {
        let n_features = data.ncols();
        let mut mean = Array1::zeros(n_features);
        let mut std = Array1::zeros(n_features);

        for j in 0..n_features {
            let col = data.column(j);
            mean[j] = col.mean().unwrap_or(0.0);
            std[j] = col.std(0.0).max(1e-8);
        }

        for mut row in data.rows_mut() {
            for j in 0..n_features {
                row[j] = (row[j] - mean[j]) / std[j];
            }
        }

        self.scaler_mean = Some(mean);
        self.scaler_std = Some(std);
    }

    /// Apply standardization using fitted parameters
    fn apply_scaling(&self, features: &mut Array1<f64>) {
        if let (Some(mean), Some(std)) = (&self.scaler_mean, &self.scaler_std) {
            for j in 0..features.len() {
                features[j] = (features[j] - mean[j]) / std[j];
            }
        }
    }

    /// Train behavior classifier from labeled data
    pub fn train_classifier(
        &mut self,
        training_data: &[(BehaviorFeatures, BehaviorType)],
    ) -> Result<()> {
        if training_data.is_empty() {
            anyhow::bail!("No training data provided");
        }

        let n_samples = training_data.len();
        let n_features = 11;
        let mut x = Array2::zeros((n_samples, n_features));
        let mut y = Array1::zeros(n_samples);

        for (i, (features, label)) in training_data.iter().enumerate() {
            let arr = Self::features_to_array(features);
            for j in 0..n_features {
                x[[i, j]] = arr[j];
            }
            y[i] = match label {
                BehaviorType::Stationary => 0,
                BehaviorType::Territorial => 1,
                BehaviorType::Linear => 2,
                BehaviorType::Random => 3,
                BehaviorType::CentralPlace => 4,
            };
        }

        self.standardize(&mut x);

        let dataset = DatasetBase::new(x, y);
        let model = DecisionTree::params()
            .max_depth(Some(10))
            .min_weight_split(2.0)
            .min_weight_leaf(1.0)
            .fit(&dataset)?;

        self.classifier = Some(model);
        Ok(())
    }

    /// Predict behavior type from features
    pub fn predict_behavior(&self, features: &BehaviorFeatures) -> Result<BehaviorPrediction> {
        let mut x = Self::features_to_array(features);
        self.apply_scaling(&mut x);

        let classifier = self
            .classifier
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Classifier not trained"))?;

        let pred = classifier.predict(&x.view().insert_axis(Axis(0)));
        let behavior = match pred[0] {
            0 => BehaviorType::Stationary,
            1 => BehaviorType::Territorial,
            2 => BehaviorType::Linear,
            3 => BehaviorType::Random,
            4 => BehaviorType::CentralPlace,
            _ => BehaviorType::Random,
        };

        Ok(BehaviorPrediction {
            behavior,
            confidence: 0.8,
            features: features.clone(),
        })
    }

    /// Train location predictor (linear regression for next position)
    pub fn train_location_predictor(&mut self, sightings: &[Sighting]) -> Result<()> {
        if sightings.len() < 3 {
            anyhow::bail!("Need at least 3 sightings for location prediction");
        }

        let mut sorted = sightings.to_vec();
        sorted.sort_by_key(|s| s.observed_on);

        let n = sorted.len() - 1;
        let mut x = Array2::zeros((n, 3));
        let mut y_lat = Array1::zeros(n);
        let mut y_lon = Array1::zeros(n);

        for i in 0..n {
            let dt = sorted[i + 1]
                .observed_on
                .signed_duration_since(sorted[i].observed_on)
                .num_seconds() as f64
                / 3600.0;
            x[[i, 0]] = dt;
            x[[i, 1]] = sorted[i].latitude;
            x[[i, 2]] = sorted[i].longitude;
            y_lat[i] = sorted[i + 1].latitude;
            y_lon[i] = sorted[i + 1].longitude;
        }

        // Train latitude model
        let dataset_lat = DatasetBase::new(x, y_lat);
        let model = LinearRegression::default().fit(&dataset_lat)?;
        self.location_model = Some(model);

        Ok(())
    }

    /// Predict next location
    pub fn predict_next_location(
        &self,
        sightings: &[Sighting],
        horizon_hours: u32,
    ) -> Result<LocationPrediction> {
        if sightings.len() < 2 {
            anyhow::bail!("Need at least 2 sightings");
        }

        let mut sorted = sightings.to_vec();
        sorted.sort_by_key(|s| s.observed_on);

        let last = sorted.last().unwrap();
        let prev = &sorted[sorted.len() - 2];

        let dt = last
            .observed_on
            .signed_duration_since(prev.observed_on)
            .num_seconds() as f64
            / 3600.0;

        // Simple linear extrapolation based on last movement
        let bearing = crate::movement::calculate_bearing(
            prev.latitude,
            prev.longitude,
            last.latitude,
            last.longitude,
        );
        let distance = crate::movement::haversine_distance(
            prev.latitude,
            prev.longitude,
            last.latitude,
            last.longitude,
        );
        let speed = if dt > 0.0 { distance / dt } else { 0.0 };

        let pred_distance = speed * horizon_hours as f64;

        // Destination point calculation
        let lat1 = last.latitude.to_radians();
        let lon1 = last.longitude.to_radians();
        let brng = bearing.to_radians();
        let d = pred_distance / 6371.0;

        let lat2 = (lat1.sin() * d.cos() + lat1.cos() * d.sin() * brng.cos()).asin();
        let lon2 =
            lon1 + (brng.sin() * d.cos() * lat1.cos()).atan2(d.cos() - lat1.sin() * lat2.sin());

        let confidence = (1.0 / (1.0 + horizon_hours as f64 * 0.1)).min(0.95);

        Ok(LocationPrediction {
            latitude: lat2.to_degrees(),
            longitude: lon2.to_degrees(),
            confidence,
            horizon_hours,
        })
    }
}

/// Predict activity patterns by hour of day
pub fn predict_activity_pattern(sightings: &[Sighting]) -> Vec<ActivityPrediction> {
    let mut hourly_counts = [0usize; 24];
    let mut total = 0;

    for s in sightings {
        let hour = s.observed_on.hour() as usize;
        hourly_counts[hour] += 1;
        total += 1;
    }

    if total == 0 {
        return vec![];
    }

    // Simple smoothing with prior
    let mut predictions = Vec::new();
    for (hour, &count) in hourly_counts.iter().enumerate() {
        let count = count as f64;
        let probability = (count + 0.5) / (total as f64 + 12.0); // Laplace smoothing
        predictions.push(ActivityPrediction {
            hour: hour as u8,
            activity_probability: probability,
            expected_sightings: probability * (total as f64 / 24.0),
        });
    }

    predictions
}

/// Generate synthetic training data for behavior classification
pub fn generate_synthetic_training_data() -> Vec<(BehaviorFeatures, BehaviorType)> {
    let mut data = Vec::new();
    let mut rng = rand::thread_rng();

    // Stationary: low speed, low variance, low straightness
    for _ in 0..50 {
        data.push((
            BehaviorFeatures {
                avg_speed_kmh: rng.gen_range(0.0..0.5),
                speed_variance: rng.gen_range(0.0..0.1),
                turning_angle_variance: rng.gen_range(0.0..1.0),
                straightness_index: rng.gen_range(0.0..0.3),
                territory_radius_km: rng.gen_range(0.0..2.0),
                time_span_hours: rng.gen_range(1.0..100.0),
                num_sightings: rng.gen_range(5..20),
                hour_of_day_sin: rng.gen_range(-1.0..1.0),
                hour_of_day_cos: rng.gen_range(-1.0..1.0),
                day_of_year_sin: rng.gen_range(-1.0..1.0),
                day_of_year_cos: rng.gen_range(-1.0..1.0),
            },
            BehaviorType::Stationary,
        ));
    }

    // Territorial: moderate speed, high turning variance, low straightness, small territory
    for _ in 0..50 {
        data.push((
            BehaviorFeatures {
                avg_speed_kmh: rng.gen_range(1.0..5.0),
                speed_variance: rng.gen_range(0.5..3.0),
                turning_angle_variance: rng.gen_range(1.0..3.0),
                straightness_index: rng.gen_range(0.1..0.4),
                territory_radius_km: rng.gen_range(5.0..20.0),
                time_span_hours: rng.gen_range(24.0..500.0),
                num_sightings: rng.gen_range(10..50),
                hour_of_day_sin: rng.gen_range(-1.0..1.0),
                hour_of_day_cos: rng.gen_range(-1.0..1.0),
                day_of_year_sin: rng.gen_range(-1.0..1.0),
                day_of_year_cos: rng.gen_range(-1.0..1.0),
            },
            BehaviorType::Territorial,
        ));
    }

    // Linear: high speed, low turning variance, high straightness
    for _ in 0..50 {
        data.push((
            BehaviorFeatures {
                avg_speed_kmh: rng.gen_range(3.0..10.0),
                speed_variance: rng.gen_range(0.5..2.0),
                turning_angle_variance: rng.gen_range(0.0..0.5),
                straightness_index: rng.gen_range(0.7..1.0),
                territory_radius_km: rng.gen_range(50.0..500.0),
                time_span_hours: rng.gen_range(10.0..200.0),
                num_sightings: rng.gen_range(5..30),
                hour_of_day_sin: rng.gen_range(-1.0..1.0),
                hour_of_day_cos: rng.gen_range(-1.0..1.0),
                day_of_year_sin: rng.gen_range(-1.0..1.0),
                day_of_year_cos: rng.gen_range(-1.0..1.0),
            },
            BehaviorType::Linear,
        ));
    }

    // Random: moderate speed, high turning variance, low straightness
    for _ in 0..50 {
        data.push((
            BehaviorFeatures {
                avg_speed_kmh: rng.gen_range(0.5..3.0),
                speed_variance: rng.gen_range(1.0..5.0),
                turning_angle_variance: rng.gen_range(2.0..4.0),
                straightness_index: rng.gen_range(0.0..0.3),
                territory_radius_km: rng.gen_range(10.0..100.0),
                time_span_hours: rng.gen_range(10.0..300.0),
                num_sightings: rng.gen_range(5..40),
                hour_of_day_sin: rng.gen_range(-1.0..1.0),
                hour_of_day_cos: rng.gen_range(-1.0..1.0),
                day_of_year_sin: rng.gen_range(-1.0..1.0),
                day_of_year_cos: rng.gen_range(-1.0..1.0),
            },
            BehaviorType::Random,
        ));
    }

    // Central place: moderate speed, low turning variance, moderate straightness, returning to center
    for _ in 0..50 {
        data.push((
            BehaviorFeatures {
                avg_speed_kmh: rng.gen_range(1.0..4.0),
                speed_variance: rng.gen_range(0.5..2.0),
                turning_angle_variance: rng.gen_range(0.5..1.5),
                straightness_index: rng.gen_range(0.3..0.6),
                territory_radius_km: rng.gen_range(5.0..30.0),
                time_span_hours: rng.gen_range(24.0..400.0),
                num_sightings: rng.gen_range(10..60),
                hour_of_day_sin: rng.gen_range(-1.0..1.0),
                hour_of_day_cos: rng.gen_range(-1.0..1.0),
                day_of_year_sin: rng.gen_range(-1.0..1.0),
                day_of_year_cos: rng.gen_range(-1.0..1.0),
            },
            BehaviorType::CentralPlace,
        ));
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Sighting, Source};
    use chrono::Utc;

    fn create_test_sightings() -> Vec<Sighting> {
        let base_time = Utc::now();
        vec![
            Sighting {
                id: Some(1),
                species: "Canis lupus".to_string(),
                scientific_name: Some("Canis lupus".to_string()),
                latitude: 45.0,
                longitude: -122.0,
                observed_on: base_time - chrono::Duration::hours(6),
                source: Source::GBIF,
                source_id: "test_1".to_string(),
                details: None,
            },
            Sighting {
                id: Some(2),
                species: "Canis lupus".to_string(),
                scientific_name: Some("Canis lupus".to_string()),
                latitude: 45.1,
                longitude: -122.1,
                observed_on: base_time - chrono::Duration::hours(3),
                source: Source::GBIF,
                source_id: "test_2".to_string(),
                details: None,
            },
            Sighting {
                id: Some(3),
                species: "Canis lupus".to_string(),
                scientific_name: Some("Canis lupus".to_string()),
                latitude: 45.2,
                longitude: -122.2,
                observed_on: base_time,
                source: Source::GBIF,
                source_id: "test_3".to_string(),
                details: None,
            },
        ]
    }

    #[test]
    fn test_extract_features() {
        let sightings = create_test_sightings();
        let features = BehaviorModel::extract_features(&sightings).unwrap();

        assert!(features.avg_speed_kmh >= 0.0);
        assert_eq!(features.num_sightings, 3);
        assert!(features.time_span_hours > 0.0);
    }

    #[test]
    fn test_predict_activity_pattern() {
        let sightings = create_test_sightings();
        let predictions = predict_activity_pattern(&sightings);

        assert_eq!(predictions.len(), 24);
        let total_prob: f64 = predictions.iter().map(|p| p.activity_probability).sum();
        assert!((total_prob - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_generate_synthetic_training_data() {
        let data = generate_synthetic_training_data();
        assert_eq!(data.len(), 250);

        let mut class_counts = std::collections::HashMap::new();
        for (_, label) in &data {
            *class_counts.entry(label.clone()).or_insert(0) += 1;
        }
        assert_eq!(class_counts.len(), 5);
    }

    #[test]
    fn test_behavior_model_train_and_predict() {
        let training_data = generate_synthetic_training_data();
        let mut model = BehaviorModel::new();
        model.train_classifier(&training_data).unwrap();

        let features = BehaviorFeatures {
            avg_speed_kmh: 0.1,
            speed_variance: 0.01,
            turning_angle_variance: 0.5,
            straightness_index: 0.1,
            territory_radius_km: 0.5,
            time_span_hours: 50.0,
            num_sightings: 10,
            hour_of_day_sin: 0.0,
            hour_of_day_cos: 1.0,
            day_of_year_sin: 0.0,
            day_of_year_cos: 1.0,
        };

        let prediction = model.predict_behavior(&features).unwrap();
        assert!(prediction.confidence > 0.0);
    }

    #[test]
    fn test_location_prediction() {
        let sightings = create_test_sightings();
        let model = BehaviorModel::new();
        let prediction = model.predict_next_location(&sightings, 1).unwrap();

        assert!(prediction.latitude >= -90.0 && prediction.latitude <= 90.0);
        assert!(prediction.longitude >= -180.0 && prediction.longitude <= 180.0);
        assert!(prediction.confidence > 0.0 && prediction.confidence <= 1.0);
        assert_eq!(prediction.horizon_hours, 1);
    }
}
