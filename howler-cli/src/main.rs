use anyhow::Result;
use clap::{Parser, Subcommand};
use howler_core::{Config, Database, Source};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "howler")]
#[command(about = "Real-world wolf tracking and sighting data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Fetch fresh data from all available sources
    #[arg(long)]
    fetch: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the TUI interface
    Tui,
    /// Launch the GUI interface
    Gui,
    /// Generate a report from cached data
    Report,
}

fn generate_report(db: &Database) -> Result<()> {
    let sightings = db.get_all_sightings()?;

    if sightings.is_empty() {
        println!("No sightings data available. Run with --fetch to populate the database.");
        return Ok(());
    }

    println!("=== Wolf Sighting Report ===\n");

    // Summary statistics
    println!("Total sightings: {}", sightings.len());

    // Group by source
    let mut by_source: HashMap<Source, usize> = HashMap::new();
    for sighting in &sightings {
        *by_source.entry(sighting.source.clone()).or_insert(0) += 1;
    }

    println!("\nBy source:");
    for (source, count) in by_source {
        println!("  {}: {}", source, count);
    }

    // Group by species
    let mut by_species: HashMap<String, usize> = HashMap::new();
    for sighting in &sightings {
        *by_species.entry(sighting.species.clone()).or_insert(0) += 1;
    }

    println!("\nBy species:");
    for (species, count) in by_species {
        println!("  {}: {}", species, count);
    }

    // Geographic range
    let min_lat = sightings
        .iter()
        .map(|s| s.latitude)
        .fold(f64::INFINITY, f64::min);
    let max_lat = sightings
        .iter()
        .map(|s| s.latitude)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_lon = sightings
        .iter()
        .map(|s| s.longitude)
        .fold(f64::INFINITY, f64::min);
    let max_lon = sightings
        .iter()
        .map(|s| s.longitude)
        .fold(f64::NEG_INFINITY, f64::max);

    println!("\nGeographic range:");
    println!("  Latitude: {:.4} to {:.4}", min_lat, max_lat);
    println!("  Longitude: {:.4} to {:.4}", min_lon, max_lon);

    // Date range
    let dates: Vec<_> = sightings.iter().map(|s| s.observed_on).collect();
    if let (Some(min_date), Some(max_date)) = (dates.iter().min(), dates.iter().max()) {
        println!("\nDate range:");
        println!("  From: {}", min_date.format("%Y-%m-%d"));
        println!("  To: {}", max_date.format("%Y-%m-%d"));
    }

    // IUCN status
    if let Some(status) = db.get_species_status("Canis lupus")? {
        println!("\nIUCN Red List Status:");
        println!("  Scientific name: {}", status.scientific_name);
        if let Some(common) = status.common_name {
            println!("  Common name: {}", common);
        }
        if let Some(category) = status.red_list_category {
            println!("  Category: {}", category);
        }
        if let Some(trend) = status.population_trend {
            println!("  Population trend: {}", trend);
        }
        if let Some(threats) = status.threats {
            println!("  Threats: {}", threats);
        }
    } else {
        println!("\nIUCN status not available (run with --fetch and IUCN_TOKEN set)");
    }

    // Recent sightings (last 5)
    println!("\nRecent sightings (last 5):");
    let mut recent = sightings.clone();
    recent.sort_by_key(|b| std::cmp::Reverse(b.observed_on));
    for (i, sighting) in recent.iter().take(5).enumerate() {
        println!(
            "  {}. {} - {} ({}) at ({:.4}, {:.4})",
            i + 1,
            sighting.species,
            sighting.observed_on.format("%Y-%m-%d"),
            sighting.source,
            sighting.latitude,
            sighting.longitude
        );
    }

    Ok(())
}

async fn fetch_data(config: &Config) -> Result<()> {
    let db = Database::new("howler.db")?;

    println!("Fetching data from available sources...");

    // GBIF (no key needed)
    println!("Fetching from GBIF...");
    match howler_core::gbif::fetch_and_cache_gbif(&db, 100).await {
        Ok(count) => println!("  GBIF: {} sightings fetched", count),
        Err(e) => eprintln!("  GBIF error: {}", e),
    }

    // Movebank (requires credentials)
    println!("Fetching from Movebank...");
    match howler_core::movebank::fetch_and_cache_movebank(&db, config, 100).await {
        Ok(count) => println!("  Movebank: {} sightings fetched", count),
        Err(e) => eprintln!("  Movebank error: {}", e),
    }

    // iNaturalist (token optional)
    println!("Fetching from iNaturalist...");
    match howler_core::inaturalist::fetch_and_cache_inaturalist(&db, config, 100).await {
        Ok(count) => println!("  iNaturalist: {} sightings fetched", count),
        Err(e) => eprintln!("  iNaturalist error: {}", e),
    }

    // IUCN (requires token)
    println!("Fetching from IUCN...");
    match howler_core::iucn::fetch_and_cache_iucn(&db, config).await {
        Ok(count) => println!("  IUCN: {} species statuses fetched", count),
        Err(e) => eprintln!("  IUCN error: {}", e),
    }

    println!("\nData fetching complete.");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::from_env();

    if cli.fetch {
        tokio::runtime::Runtime::new()?.block_on(fetch_data(&config))?;
    }

    match cli.command {
        Some(Commands::Tui) => {
            // Delegate to howler-tui
            howler_tui::run()
        }
        Some(Commands::Gui) => {
            // Delegate to howler-gui
            howler_gui::run()
        }
        Some(Commands::Report) => {
            let db = Database::new("howler.db")?;
            generate_report(&db)
        }
        None => {
            // Default to TUI
            howler_tui::run()
        }
    }
}
