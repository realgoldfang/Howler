// shared/types.ts - Common data types for both platforms
export interface Sighting {
  id?: number;
  species: string;
  scientificName?: string;
  latitude: number;
  longitude: number;
  observedOn: string;
  source: string;
  sourceId?: string;
  details?: string;
  distanceKm?: number;
}

export interface SightingFilter {
  startDate?: string;
  endDate?: string;
  source?: string[];
  species?: string;
  minLatitude?: number;
  maxLatitude?: number;
  minLongitude?: number;
  maxLongitude?: number;
  search?: string;
  limit?: number;
  offset?: number;
}

export interface MovementAnalysis {
  movements: Movement[];
  averageSpeedKmh: number;
  maximumSpeedKmh: number;
  totalDistanceKm: number;
  pattern: MovementPattern;
  averageBearingDegrees: number;
}

export interface Movement {
  fromId: number;
  toId: number;
  distanceKm: number;
  bearingDegrees: number;
  durationSeconds: number;
  speedKmh: number;
}

export enum MovementPattern {
  Random = 'Random',
  Circular = 'Circular',
  Linear = 'Linear',
  Stationary = 'Stationary'
}

export interface TemporalAnalysis {
  hourlyActivity: ActivityStats[];
  dailyActivity: ActivityStats[];
  monthlyActivity: ActivityStats[];
  seasonalActivity: ActivityStats[];
  mostActivePeriod: string;
  leastActivePeriod: string;
}

export interface ActivityStats {
  period: string;
  count: number;
  percentage: number;
}

export interface PackTerritory {
  id: string;
  sightingIds: number[];
  center: { lat: number; lon: number };
  areaKm2?: number;
  boundary?: string;
}

export interface ApiResponse<T> {
  data: T;
  pagination?: {
    total: number;
    limit: number;
    offset: number;
    hasMore: boolean;
  };
}
