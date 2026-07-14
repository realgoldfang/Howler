import axios from 'axios'
import type {
  Sighting,
  SightingCreate,
  SightingQuery,
  PaginatedResponse,
  MovementAnalysis,
  TemporalAnalysis,
  TerritoryCluster,
  DashboardStats,
} from './types'

const api = axios.create({
  baseURL: '/api',
  headers: { 'Content-Type': 'application/json' },
})

export const sightingsApi = {
  list: (query?: SightingQuery) =>
    api.get<PaginatedResponse<Sighting>>('/sightings', { params: query }).then(r => r.data),

  get: (id: number) =>
    api.get<Sighting>(`/sightings/${id}`).then(r => r.data),

  create: (data: SightingCreate) =>
    api.post<Sighting>('/sightings', data).then(r => r.data),
}

export const analysisApi = {
  movement: (sighting_ids: number[]) =>
    api.post<MovementAnalysis>('/analysis/movement', { sighting_ids }).then(r => r.data),

  temporal: (sighting_ids: number[]) =>
    api.post<TemporalAnalysis>('/analysis/temporal', { sighting_ids }).then(r => r.data),
}

export const clusteringApi = {
  territories: (sighting_ids: number[]) =>
    api.post<TerritoryCluster[]>('/clustering/territories', { sighting_ids }).then(r => r.data),
}

export const exportApi = {
  download: (format: 'csv' | 'geojson' | 'kml') =>
    api.get(`/export/${format}`, { responseType: 'blob' }).then(r => r.data),
}

export const importApi = {
  upload: (file: File) => {
    const form = new FormData()
    form.append('file', file)
    return api.post<{ imported: number; errors: string[] }>('/import', form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    }).then(r => r.data)
  },
}

export const dashboardApi = {
  stats: () =>
    api.get<PaginatedResponse<Sighting>>('/sightings', { params: { per_page: 1 } }).then(r => {
      const total = r.data.total
      return {
        total_sightings: total,
        total_wolves: 0,
        total_packs: 0,
        recent_sightings: 0,
        avg_daily_sightings: 0,
      } as DashboardStats
    }),
}

export default api
