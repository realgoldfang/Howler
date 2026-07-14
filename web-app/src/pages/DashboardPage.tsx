import { useState } from 'react'
import type { Sighting } from '../types'
import StatsCard from '../components/StatsCard'
import SightingTable from '../components/SightingTable'
import SightingModal from '../components/SightingModal'
import ConnectionStatus from '../components/ConnectionStatus'
import { useAutoFetch } from '../hooks/useAutoFetch'
import { useWebSocket } from '../hooks/useWebSocket'

export default function DashboardPage() {
  const [selectedSighting, setSelectedSighting] = useState<Sighting | null>(null)
  const { lastFetchTime, data, isLoading } = useAutoFetch(['sightings', 'dashboard'])
  const { isConnected } = useWebSocket(['sightings'])

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

  const getRelativeTime = (date: Date | null) => {
    if (!date) return 'Never'
    const seconds = Math.floor((Date.now() - date.getTime()) / 1000)
    if (seconds < 60) return `${seconds} seconds ago`
    const minutes = Math.floor(seconds / 60)
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '24px' }}>
        <div>
          <h1 style={{ fontSize: '24px', fontWeight: 700 }}>Dashboard</h1>
          <p style={{ fontSize: '14px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Overview of wolf sighting activity
          </p>
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '8px' }}>
          <ConnectionStatus isConnected={isConnected} />
          <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
            Last updated: {getRelativeTime(lastFetchTime)}
          </span>
        </div>
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
