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
}

export interface Movement {
  from: { latitude: number; longitude: number; timestamp: string };
  to: { latitude: number; longitude: number; timestamp: string };
  distanceKm: number;
  speedKmh: number;
}

export interface MovementAnalysis {
  movements: Movement[];
  averageSpeedKmh: number;
  totalDistanceKm: number;
  pattern: string;
}

export interface ActivityStats {
  period: string;
  count: number;
  percentage: number;
}

export interface TemporalAnalysis {
  hourlyActivity: ActivityStats[];
  mostActivePeriod: string;
}

export interface Territory {
  id: number;
  centroid: { latitude: number; longitude: number };
  radiusKm: number;
  sightingCount: number;
  species: string[];
}

export interface FilterParams {
  startDate?: string;
  endDate?: string;
  source?: string;
  species?: string;
  limit?: number;
  offset?: number;
}

export type RootTabParamList = {
  Home: undefined;
  Map: undefined;
  Analysis: undefined;
  Settings: undefined;
};
