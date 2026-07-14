export interface Sighting {
  id: number
  latitude: number
  longitude: number
  timestamp: string
  wolf_id?: string
  pack?: string
  behavior?: string
  health?: string
  count: number
  notes?: string
  photo_url?: string
  created_at: string
  updated_at: string
}

export interface SightingCreate {
  latitude: number
  longitude: number
  timestamp?: string
  wolf_id?: string
  pack?: string
  behavior?: string
  health?: string
  count?: number
  notes?: string
  photo_url?: string
}

export interface SightingQuery {
  page?: number
  per_page?: number
  wolf_id?: string
  pack?: string
  behavior?: string
  start_date?: string
  end_date?: string
  min_lat?: number
  max_lat?: number
  min_lon?: number
  max_lon?: number
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export interface MovementAnalysis {
  sighting_ids: number[]
  total_distance_km: number
  avg_speed_kmh: number
  max_speed_kmh: number
  segments: MovementSegment[]
  convex_hull?: { type: string; coordinates: number[][][] }
}

export interface MovementSegment {
  from_id: number
  to_id: number
  distance_km: number
  duration_hours: number
  speed_kmh: number
}

export interface TemporalAnalysis {
  sighting_ids: number[]
  hourly_distribution: number[]
  daily_distribution: number[]
  monthly_distribution: number[]
  peak_hour: number
  peak_day: number
}

export interface TerritoryCluster {
  id: number
  center_lat: number
  center_lon: number
  radius_km: number
  sighting_count: number
  sighting_ids: number[]
}

export interface DashboardStats {
  total_sightings: number
  total_wolves: number
  total_packs: number
  recent_sightings: number
  avg_daily_sightings: number
}

export interface ExportFormat {
  format: 'csv' | 'geojson' | 'kml'
}

export type Theme = 'light' | 'dark'

export interface AppSettings {
  theme: Theme
  api_base_url: string
  map_default_center: [number, number]
  map_default_zoom: number
}
