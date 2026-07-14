import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { sightingsApi, clusteringApi } from '../api'
import type { Sighting, TerritoryCluster } from '../types'

export default function MapPage() {
  const [showClusters, setShowClusters] = useState(true)
  const [selectedWolf, setSelectedWolf] = useState<string>('')
  const [hoveredSighting, setHoveredSighting] = useState<Sighting | null>(null)

  const { data } = useQuery({
    queryKey: ['sightings', 'map'],
    queryFn: () => sightingsApi.list({ per_page: 500 }),
  })

  const sightings = data?.items ?? []
  const wolfIds = [...new Set(sightings.filter((s) => s.wolf_id).map((s) => s.wolf_id!))]

  const filtered = selectedWolf
    ? sightings.filter((s) => s.wolf_id === selectedWolf)
    : sightings

  const { data: clusters } = useQuery({
    queryKey: ['clusters', showClusters],
    queryFn: () => clusteringApi.territories(filtered.map((s) => s.id)),
    enabled: showClusters && filtered.length > 0,
  })

  const bounds = filtered.reduce(
    (acc, s) => ({
      minLat: Math.min(acc.minLat, s.latitude),
      maxLat: Math.max(acc.maxLat, s.latitude),
      minLon: Math.min(acc.minLon, s.longitude),
      maxLon: Math.max(acc.maxLon, s.longitude),
    }),
    { minLat: 90, maxLat: -90, minLon: 180, maxLon: -180 },
  )

  const toSvg = (lat: number, lon: number) => {
    const padding = 40
    const width = 800
    const height = 500
    const x = padding + ((lon - bounds.minLon) / (bounds.maxLon - bounds.minLon || 1)) * (width - padding * 2)
    const y = padding + ((bounds.maxLat - lat) / (bounds.maxLat - bounds.minLat || 1)) * (height - padding * 2)
    return { x, y }
  }

  return (
    <div>
      <div style={{ marginBottom: '24px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px' }}>
        <div>
          <h1 style={{ fontSize: '24px', fontWeight: 700 }}>Map View</h1>
          <p style={{ fontSize: '14px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Geographic distribution of wolf sightings
          </p>
        </div>

        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          <select
            value={selectedWolf}
            onChange={(e) => setSelectedWolf(e.target.value)}
            style={{ width: '180px' }}
          >
            <option value="">All Wolves</option>
            {wolfIds.map((id) => (
              <option key={id} value={id}>{id}</option>
            ))}
          </select>

          <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '14px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showClusters}
              onChange={(e) => setShowClusters(e.target.checked)}
              style={{ width: 'auto' }}
            />
            Territories
          </label>
        </div>
      </div>

      <div
        style={{
          background: 'var(--bg-secondary)',
          border: '1px solid var(--border-color)',
          borderRadius: 'var(--radius-lg)',
          padding: '20px',
          boxShadow: 'var(--shadow-sm)',
          position: 'relative',
        }}
      >
        {filtered.length === 0 ? (
          <div style={{ padding: '80px', textAlign: 'center', color: 'var(--text-muted)' }}>
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>🗺️</div>
            <p>No sighting data to display. Add sightings or adjust filters.</p>
          </div>
        ) : (
          <svg viewBox="0 0 800 500" style={{ width: '100%', height: 'auto' }}>
            <rect width="800" height="500" fill="var(--bg-tertiary)" rx="8" />

            <defs>
              <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
                <path d="M 40 0 L 0 0 0 40" fill="none" stroke="var(--border-color)" strokeWidth="0.5" />
              </pattern>
            </defs>
            <rect width="800" height="500" fill="url(#grid)" />

            {showClusters && clusters?.map((c: TerritoryCluster) => {
              const center = toSvg(c.center_lat, c.center_lon)
              const radiusScale = 40000
              return (
                <g key={c.id}>
                  <circle
                    cx={center.x}
                    cy={center.y}
                    r={c.radius_km * radiusScale}
                    fill="var(--accent-light)"
                    fillOpacity="0.3"
                    stroke="var(--accent)"
                    strokeWidth="1.5"
                    strokeDasharray="6 3"
                  />
                  <text
                    x={center.x}
                    y={center.y - c.radius_km * radiusScale - 8}
                    textAnchor="middle"
                    fill="var(--text-muted)"
                    fontSize="11"
                    fontWeight="500"
                  >
                    Territory {c.id} ({c.sighting_count} sightings)
                  </text>
                </g>
              )
            })}

            {filtered.map((s) => {
              const pos = toSvg(s.latitude, s.longitude)
              const color = s.wolf_id
                ? `hsl(${(s.wolf_id.charCodeAt(0) * 37) % 360}, 70%, 50%)`
                : 'var(--accent)'
              return (
                <g key={s.id}>
                  <circle
                    cx={pos.x}
                    cy={pos.y}
                    r="5"
                    fill={color}
                    stroke="white"
                    strokeWidth="1.5"
                    style={{ cursor: 'pointer', transition: 'r 0.15s' }}
                    onMouseEnter={() => setHoveredSighting(s)}
                    onMouseLeave={() => setHoveredSighting(null)}
                  />
                </g>
              )
            })}

            {hoveredSighting && (() => {
              const pos = toSvg(hoveredSighting.latitude, hoveredSighting.longitude)
              return (
                <g>
                  <rect
                    x={pos.x + 10}
                    y={pos.y - 40}
                    width="160"
                    height="50"
                    rx="6"
                    fill="var(--bg-secondary)"
                    stroke="var(--border-color)"
                    strokeWidth="1"
                  />
                  <text x={pos.x + 18} y={pos.y - 22} fill="var(--text-primary)" fontSize="12" fontWeight="600">
                    {hoveredSighting.wolf_id || `Sighting #${hoveredSighting.id}`}
                  </text>
                  <text x={pos.x + 18} y={pos.y - 6} fill="var(--text-muted)" fontSize="11">
                    {hoveredSighting.pack || 'Unknown pack'}
                  </text>
                </g>
              )
            })()}
          </svg>
        )}

        <div style={{ marginTop: '16px', display: 'flex', gap: '16px', flexWrap: 'wrap' }}>
          <div style={{ fontSize: '13px', color: 'var(--text-muted)' }}>
            Showing {filtered.length} of {sightings.length} sightings
          </div>
          {clusters && clusters.length > 0 && (
            <div style={{ fontSize: '13px', color: 'var(--text-muted)' }}>
              {clusters.length} territor{clusters.length === 1 ? 'y' : 'ies'} detected
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
