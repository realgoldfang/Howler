use crate::models::Sighting;
use anyhow::Result;
use chrono::{Datelike, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time period for temporal analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimePeriod {
    Hour(u8),           // 0-23
    DayOfWeek(Weekday), // Monday-Sunday
    Month(u8),          // 1-12
    Season,             // Winter, Spring, Summer, Fall
}

/// Activity statistics for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStats {
    /// Time period
    pub period: String,
    /// Number of sightings in this period
    pub count: usize,
    /// Percentage of total sightings
    pub percentage: f64,
}

/// Temporal analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysis {
    /// Activity by hour of day
    pub hourly_activity: Vec<ActivityStats>,
    /// Activity by day of week
    pub daily_activity: Vec<ActivityStats>,
    /// Activity by month
    pub monthly_activity: Vec<ActivityStats>,
    /// Activity by season
    pub seasonal_activity: Vec<ActivityStats>,
    /// Most active time period
    pub most_active_period: String,
    /// Least active time period
    pub least_active_period: String,
}

/// Season classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Season {
    Winter,
    Spring,
    Summer,
    Fall,
}

/// Get season from month
pub fn month_to_season(month: u8) -> Season {
    match month {
        12 | 1 | 2 => Season::Winter,
        3..=5 => Season::Spring,
        6..=8 => Season::Summer,
        9..=11 => Season::Fall,
        _ => Season::Winter,
    }
}

/// Analyze temporal patterns in sightings
pub fn analyze_temporal_patterns(sightings: &[Sighting]) -> Result<TemporalAnalysis> {
    if sightings.is_empty() {
        return Ok(TemporalAnalysis {
            hourly_activity: vec![],
            daily_activity: vec![],
            monthly_activity: vec![],
            seasonal_activity: vec![],
            most_active_period: "No data".to_string(),
            least_active_period: "No data".to_string(),
        });
    }

    let total = sightings.len();

    // Analyze by hour
    let mut hour_counts: HashMap<u8, usize> = HashMap::new();
    for sighting in sightings {
        let hour = sighting.observed_on.hour() as u8;
        *hour_counts.entry(hour).or_insert(0) += 1;
    }

    let mut hourly_activity: Vec<ActivityStats> = hour_counts
        .into_iter()
        .map(|(hour, count)| ActivityStats {
            period: format!("{}:00", hour),
            count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();
    hourly_activity.sort_by(|a, b| a.period.cmp(&b.period));

    // Analyze by day of week
    let mut day_counts: HashMap<Weekday, usize> = HashMap::new();
    for sighting in sightings {
        let weekday = sighting.observed_on.weekday();
        *day_counts.entry(weekday).or_insert(0) += 1;
    }

    let mut daily_activity: Vec<ActivityStats> = day_counts
        .into_iter()
        .map(|(day, count)| ActivityStats {
            period: format!("{:?}", day),
            count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();
    daily_activity.sort_by(|a, b| a.period.cmp(&b.period));

    // Analyze by month
    let mut month_counts: HashMap<u8, usize> = HashMap::new();
    for sighting in sightings {
        let month = sighting.observed_on.month() as u8;
        *month_counts.entry(month).or_insert(0) += 1;
    }

    let mut monthly_activity: Vec<ActivityStats> = month_counts
        .into_iter()
        .map(|(month, count)| ActivityStats {
            period: format!("Month {}", month),
            count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();
    monthly_activity.sort_by(|a, b| a.period.cmp(&b.period));

    // Analyze by season
    let mut season_counts: HashMap<Season, usize> = HashMap::new();
    for sighting in sightings {
        let month = sighting.observed_on.month() as u8;
        let season = month_to_season(month);
        *season_counts.entry(season).or_insert(0) += 1;
    }

    let mut seasonal_activity: Vec<ActivityStats> = season_counts
        .into_iter()
        .map(|(season, count)| ActivityStats {
            period: format!("{:?}", season),
            count,
            percentage: (count as f64 / total as f64) * 100.0,
        })
        .collect();
    seasonal_activity.sort_by(|a, b| a.period.cmp(&b.period));

    // Find most and least active periods
    let all_periods: Vec<&ActivityStats> = hourly_activity
        .iter()
        .chain(daily_activity.iter())
        .chain(monthly_activity.iter())
        .chain(seasonal_activity.iter())
        .collect();

    let most_active = all_periods
        .iter()
        .max_by(|a, b| a.count.cmp(&b.count))
        .map(|s| s.period.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let least_active = all_periods
        .iter()
        .filter(|s| s.count > 0)
        .min_by(|a, b| a.count.cmp(&b.count))
        .map(|s| s.period.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(TemporalAnalysis {
        hourly_activity,
        daily_activity,
        monthly_activity,
        seasonal_activity,
        most_active_period: most_active,
        least_active_period: least_active,
    })
}

/// Calculate trend over time (increasing, decreasing, stable)
pub fn calculate_trend(sightings: &[Sighting]) -> String {
    if sightings.len() < 2 {
        return "Insufficient data".to_string();
    }

    let mut sorted_sightings = sightings.to_vec();
    sorted_sightings.sort_by_key(|a| a.observed_on);

    // Group by month
    let mut monthly_counts: HashMap<String, usize> = HashMap::new();
    for sighting in &sorted_sightings {
        let key = sighting.observed_on.format("%Y-%m").to_string();
        *monthly_counts.entry(key).or_insert(0) += 1;
    }

    let mut counts: Vec<usize> = monthly_counts.values().cloned().collect();
    if counts.len() < 2 {
        return "Insufficient data".to_string();
    }

    counts.sort();

    let first_half: f64 = counts[..counts.len() / 2].iter().sum::<usize>() as f64;
    let second_half: f64 = counts[counts.len() / 2..].iter().sum::<usize>() as f64;

    let ratio = if first_half > 0.0 {
        second_half / first_half
    } else {
        1.0
    };

    if ratio > 1.2 {
        "Increasing".to_string()
    } else if ratio < 0.8 {
        "Decreasing".to_string()
    } else {
        "Stable".to_string()
    }
}

/// Generate heat map data by time period
pub fn generate_heatmap_data(sightings: &[Sighting], period: TimePeriod) -> Vec<(String, usize)> {
    let mut data: HashMap<String, usize> = HashMap::new();

    for sighting in sightings {
        let key = match period {
            TimePeriod::Hour(_) => format!("{}:00", sighting.observed_on.hour()),
            TimePeriod::DayOfWeek(_) => format!("{:?}", sighting.observed_on.weekday()),
            TimePeriod::Month(_) => format!("Month {}", sighting.observed_on.month()),
            TimePeriod::Season => {
                let season = month_to_season(sighting.observed_on.month() as u8);
                format!("{:?}", season)
            }
        };

        *data.entry(key).or_insert(0) += 1;
    }

    let mut result: Vec<(String, usize)> = data.into_iter().collect();
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
    fn test_month_to_season() {
        assert_eq!(month_to_season(1), Season::Winter);
        assert_eq!(month_to_season(4), Season::Spring);
        assert_eq!(month_to_season(7), Season::Summer);
        assert_eq!(month_to_season(10), Season::Fall);
    }

    #[test]
    fn test_analyze_temporal_patterns_empty() {
        let sightings = vec![];
        let result = analyze_temporal_patterns(&sightings).unwrap();
        assert_eq!(result.hourly_activity.len(), 0);
    }

    #[test]
    fn test_analyze_temporal_patterns() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 24),
            create_test_sighting(45.1, -122.0, 2, 48),
            create_test_sighting(45.2, -122.0, 3, 72),
        ];

        let result = analyze_temporal_patterns(&sightings).unwrap();
        assert!(!result.hourly_activity.is_empty());
        assert!(!result.daily_activity.is_empty());
    }

    #[test]
    fn test_calculate_trend_empty() {
        let sightings = vec![];
        let trend = calculate_trend(&sightings);
        assert_eq!(trend, "Insufficient data");
    }

    #[test]
    fn test_calculate_trend_single() {
        let sightings = vec![create_test_sighting(45.0, -122.0, 1, 0)];
        let trend = calculate_trend(&sightings);
        assert_eq!(trend, "Insufficient data");
    }

    #[test]
    fn test_calculate_trend_stable() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 720),
            create_test_sighting(45.1, -122.0, 2, 360),
            create_test_sighting(45.2, -122.0, 3, 0),
        ];

        let trend = calculate_trend(&sightings);
        assert!(!trend.is_empty());
    }

    #[test]
    fn test_generate_heatmap_data() {
        let sightings = vec![
            create_test_sighting(45.0, -122.0, 1, 24),
            create_test_sighting(45.1, -122.0, 2, 48),
        ];

        let data = generate_heatmap_data(&sightings, TimePeriod::Hour(0));
        assert!(!data.is_empty());
    }
}
