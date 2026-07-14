import axios from 'axios';
import type { FilterParams, Sighting, MovementAnalysis, TemporalAnalysis, Territory } from './types';

const API_BASE = 'http://localhost:8080';

const client = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
});

export async function fetchSightings(params?: FilterParams): Promise<Sighting[]> {
  const { data } = await client.get('/api/sightings', { params });
  return data;
}

export async function fetchSighting(id: number): Promise<Sighting> {
  const { data } = await client.get(`/api/sightings/${id}`);
  return data;
}

export async function createSighting(sighting: Omit<Sighting, 'id'>): Promise<Sighting> {
  const { data } = await client.post('/api/sightings', sighting);
  return data;
}

export async function analyzeMovement(sightingIds: number[]): Promise<MovementAnalysis> {
  const { data } = await client.post('/api/analysis/movement', { sighting_ids: sightingIds });
  return data;
}

export async function analyzeTemporal(sightingIds: number[]): Promise<TemporalAnalysis> {
  const { data } = await client.post('/api/analysis/temporal', { sighting_ids: sightingIds });
  return data;
}

export async function fetchTerritories(sightingIds: number[]): Promise<Territory[]> {
  const { data } = await client.post('/api/clustering/territories', { sighting_ids: sightingIds });
  return data;
}

export async function exportData(format: 'csv' | 'geojson' | 'kml'): Promise<string> {
  const { data } = await client.get(`/api/export/${format}`);
  return typeof data === 'string' ? data : JSON.stringify(data);
}

export async function importData(file: { uri: string; name: string; type: string }): Promise<{ imported: number }> {
  const form = new FormData();
  form.append('file', { uri: file.uri, name: file.name, type: file.type } as any);
  const { data } = await client.post('/api/import', form, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
  return data;
}
