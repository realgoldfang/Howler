import axios from 'axios';
import type { Sighting } from './types';

export const API_BASE = 'http://localhost:8080';

const client = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
});

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

async function unwrap<T>(promise: Promise<any>): Promise<T> {
  const response = await promise;
  const res: ApiResponse<T> = response.data;
  if (!res.success || res.data === null) {
    throw new Error(res.error || 'Unknown API error');
  }
  return res.data;
}

export async function fetchSightings(): Promise<Sighting[]> {
  return unwrap<Sighting[]>(client.get('/api/sightings'));
}

export async function fetchSighting(id: number): Promise<Sighting> {
  return unwrap<Sighting>(client.get(`/api/sightings/${id}`));
}

export async function createSighting(sighting: Omit<Sighting, 'id'>): Promise<number> {
  return unwrap<number>(client.post('/api/sightings', {
    species: sighting.species,
    scientific_name: sighting.scientificName,
    latitude: sighting.latitude,
    longitude: sighting.longitude,
    source: sighting.source,
    source_id: sighting.sourceId || '',
    details: sighting.details,
  }));
}

export async function updateSighting(id: number, sighting: Omit<Sighting, 'id'>): Promise<number> {
  return unwrap<number>(client.put(`/api/sightings/${id}`, {
    species: sighting.species,
    scientific_name: sighting.scientificName,
    latitude: sighting.latitude,
    longitude: sighting.longitude,
    source: sighting.source,
    source_id: sighting.sourceId || '',
    details: sighting.details,
  }));
}

export async function analyzeSighting(id: number): Promise<string> {
  return unwrap<string>(client.post(`/api/analysis/${id}`, {}));
}

export async function exportData(format: 'json'): Promise<unknown> {
  return unwrap<unknown>(client.get(`/api/export/${format}`));
}
