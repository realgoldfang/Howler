import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { sightingsApi } from '../api'
import type { Sighting } from '../types'
import StatsCard from '../components/StatsCard'
import SightingTable from '../components/SightingTable'
import SightingModal from '../components/SightingModal'

export default function DashboardPage() {
  const [selectedSighting, setSelectedSighting] = useState<Sighting | null>(null)

  const { data, isLoading } = useQuery({
    queryKey: ['sightings', 'dashboard'],
    queryFn: () => sightingsApi.list({ per_page: 50, sort_by: 'timestamp', sort_dir: 'desc' }),
  })

  const sightings = data?.items ?? []

  const stats = {
    total: data?.total ?? 0,
    uniqueWolves: new Set(sightings.filter((s) => s.wolf_id).map((s) => s.wolf_id)).size,
    uniquePacks: new Set(sightings.filter((s) => s.pack).map((s) => s.pack)).size,
    recentWeek: sightings.filter((s) => {
      const d = new Date(s.timestamp)
      const now = new Date()
      return now.getTime() - d.getTime() < 7 * 24 * 60 * 60 * 1000
    }).length,
  }

  return (
    <div>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '24px', fontWeight: 700 }}>Dashboard</h1>
        <p style={{ fontSize: '14px', color: 'var(--text-muted)', marginTop: '4px' }}>
          Overview of wolf sighting activity
        </p>
      </div>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
          gap: '16px',
          marginBottom: '24px',
        }}
      >
        <StatsCard title="Total Sightings" value={stats.total} icon="📍" />
        <StatsCard title="Unique Wolves" value={stats.uniqueWolves} icon="🐺" />
        <StatsCard title="Active Packs" value={stats.uniquePacks} icon="🐾" />
        <StatsCard title="This Week" value={stats.recentWeek} icon="📅" />
      </div>

      <div style={{ marginBottom: '24px' }}>
        <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>
          Recent Sightings
        </h2>
        <SightingTable
          sightings={sightings}
          onRowClick={setSelectedSighting}
          loading={isLoading}
        />
      </div>

      {selectedSighting && (
        <SightingModal
          sighting={selectedSighting}
          onClose={() => setSelectedSighting(null)}
        />
      )}
    </div>
  )
}
