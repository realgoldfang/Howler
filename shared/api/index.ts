// shared/api/index.ts - API client for both platforms
import { Sighting, SightingFilter, MovementAnalysis, TemporalAnalysis, PackTerritory, MovementPattern } from './types';

export class HowlerAPI {
  private baseUrl: string;
  private token?: string;

  constructor(baseUrl: string = 'http://localhost:8080', token?: string) {
    this.baseUrl = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
    this.token = token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(this.token && { Authorization: `Bearer ${this.token}` }),
    };

    const config: RequestInit = {
      headers,
      ...options,
    };

    try {
      const response = await fetch(url, config);
      
      if (!response.ok) {
        throw new Error(`API Error: ${response.status} ${response.statusText}`);
      }

      // Handle empty responses
      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return response.json();
      }
      return response.text() as unknown as T;
    } catch (error) {
      console.error('API Request failed:', error);
      throw error;\n    }
  }

  // Sightings API
  async getSightings(filters?: SightingFilter): Promise<{ data: Sighting[], pagination?: any }> {
    const params = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== '') {
          params.append(key, value.toString());
        }
      });
    }

    const query = params.toString();
    const endpoint = `/api/sightings${query ? `?${query}` : ''}`;

    return this.request<any>(endpoint);
  }

  async getSighting(id: number): Promise<Sighting> {
    return this.request<Sighting>(`/api/sightings/${id}`);
  }

  async createSighting(sighting: Partial<Sighting>): Promise<Sighting> {
    return this.request<Sighting>('/api/sightings', {
      method: 'POST',
      body: JSON.stringify(sighting),
    });
  }

  // Analysis endpoints
  async getMovementAnalysis(sightingIds: number[]): Promise<MovementAnalysis> {
    return this.request<MovementAnalysis>('/api/analysis/movement', {
      method: 'POST',
      body: JSON.stringify({ sighting_ids: sightingIds }),
    });
  }

  async getTemporalAnalysis(sightingIds: number[]): Promise<TemporalAnalysis> {
    return this.request<TemporalAnalysis>('/api/analysis/temporal', {
      method: 'POST',
      body: JSON.stringify({ sighting_ids: sightingIds }),
    });
  }

  // Clustering/territories
  async getPackTerritories(sightingIds: number[]): Promise<PackTerritory[]> {
    return this.request<PackTerritory[]>('/api/clustering/territories', {
      method: 'POST',
      body: JSON.stringify({ sighting_ids: sightingIds }),
    });
  }

  // Export endpoints
  async exportData(format: 'csv' | 'geojson' | 'kml', filters?: SightingFilter): Promise<Blob> {
    const params = new URLSearchParams();
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== '') {
          params.append(key, value.toString());
        }
      });
    }

    const query = params.toString();
    const endpoint = `/api/export/${format}${query ? `?${query}` : ''}`;

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...(this.token && { Authorization: `Bearer ${this.token}` }),
      },
    });

    if (!response.ok) {
      throw new Error(`Export failed: ${response.statusText}`);
    }

    return response.blob();
  }

  // Health check
  async healthCheck(): Promise<any> {
    return this.request<any>('/health');
  }
}

export default HowlerAPI;
