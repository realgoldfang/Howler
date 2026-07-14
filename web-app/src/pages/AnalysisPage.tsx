import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { sightingsApi, analysisApi } from '../api'
import BarChart from '../components/BarChart'
import StatsCard from '../components/StatsCard'

export default function AnalysisPage() {
  const [selectedIds, setSelectedIds] = useState<number[]>([])

  const { data } = useQuery({
    queryKey: ['sightings', 'analysis'],
    queryFn: () => sightingsApi.list({ per_page: 200 }),
  })

  const sightings = data?.items ?? []

  const { data: movement, isLoading: movementLoading } = useQuery({
    queryKey: ['movement', selectedIds],
    queryFn: () => analysisApi.movement(selectedIds),
    enabled: selectedIds.length >= 2,
  })

  const { data: temporal, isLoading: temporalLoading } = useQuery({
    queryKey: ['temporal', selectedIds],
    queryFn: () => analysisApi.temporal(selectedIds),
    enabled: selectedIds.length >= 2,
  })

  const hourlyData = useMemo(() => {
    if (!temporal) return []
    return temporal.hourly_distribution.map((val, i) => ({
      label: `${String(i).padStart(2, '0')}`,
      value: val,
    }))
  }, [temporal])

  const dailyData = useMemo(() => {
    if (!temporal) return []
    const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
    return temporal.daily_distribution.map((val, i) => ({
      label: days[i],
      value: val,
    }))
  }, [temporal])

  const monthlyData = useMemo(() => {
    if (!temporal) return []
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
    return temporal.monthly_distribution.map((val, i) => ({
      label: months[i],
      value: val,
    }))
  }, [temporal])

  const handleSelectAll = () => {
    setSelectedIds(sightings.map((s) => s.id))
  }

  const handleSelectRecent = (count: number) => {
    const recent = sightings
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, count)
    setSelectedIds(recent.map((s) => s.id))
  }

  const handleToggle = (id: number) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    )
  }

  return (
    <div>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '24px', fontWeight: 700 }}>Analysis</h1>
        <p style={{ fontSize: '14px', color: 'var(--text-muted)', marginTop: '4px' }}>
          Movement patterns and temporal distribution analysis
        </p>
      </div>

      <div
        style={{
          background: 'var(--bg-secondary)',
          border: '1px solid var(--border-color)',
          borderRadius: 'var(--radius-lg)',
          padding: '20px',
          marginBottom: '24px',
          boxShadow: 'var(--shadow-sm)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
          <h3 style={{ fontSize: '15px', fontWeight: 600 }}>
            Select Sightings ({selectedIds.length} selected)
          </h3>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button className="secondary" onClick={handleSelectAll} style={{ fontSize: '13px', padding: '4px 12px' }}>
              Select All
            </button>
            <button className="secondary" onClick={() => handleSelectRecent(10)} style={{ fontSize: '13px', padding: '4px 12px' }}>
              Recent 10
            </button>
            <button className="secondary" onClick={() => handleSelectRecent(50)} style={{ fontSize: '13px', padding: '4px 12px' }}>
              Recent 50
            </button>
            <button className="secondary" onClick={() => setSelectedIds([])} style={{ fontSize: '13px', padding: '4px 12px' }}>
              Clear
            </button>
          </div>
        </div>

        <div
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: '6px',
            maxHeight: '120px',
            overflowY: 'auto',
            padding: '8px',
            background: 'var(--bg-tertiary)',
            borderRadius: 'var(--radius-sm)',
          }}
        >
          {sightings.map((s) => (
            <button
              key={s.id}
              onClick={() => handleToggle(s.id)}
              style={{
                padding: '4px 10px',
                fontSize: '12px',
                borderRadius: '12px',
                background: selectedIds.includes(s.id) ? 'var(--accent)' : 'var(--bg-secondary)',
                color: selectedIds.includes(s.id) ? 'white' : 'var(--text-secondary)',
                border: '1px solid',
                borderColor: selectedIds.includes(s.id) ? 'var(--accent)' : 'var(--border-color)',
                fontWeight: 500,
              }}
            >
              #{s.id}
            </button>
          ))}
        </div>
      </div>

      {selectedIds.length < 2 ? (
        <div
          style={{
            padding: '60px',
            textAlign: 'center',
            color: 'var(--text-muted)',
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-color)',
            borderRadius: 'var(--radius-lg)',
          }}
        >
          <div style={{ fontSize: '48px', marginBottom: '16px' }}>📈</div>
          <p>Select at least 2 sightings to run analysis</p>
        </div>
      ) : (
        <>
          {movement && (
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
                gap: '16px',
                marginBottom: '24px',
              }}
            >
              <StatsCard
                title="Total Distance"
                value={`${movement.total_distance_km.toFixed(1)} km`}
                icon="📏"
              />
              <StatsCard
                title="Avg Speed"
                value={`${movement.avg_speed_kmh.toFixed(1)} km/h`}
                icon="⚡"
              />
              <StatsCard
                title="Max Speed"
                value={`${movement.max_speed_kmh.toFixed(1)} km/h`}
                icon="🚀"
              />
              <StatsCard
                title="Segments"
                value={movement.segments.length}
                icon="🔗"
              />
            </div>
          )}

          {movementLoading && (
            <p style={{ color: 'var(--text-muted)', textAlign: 'center', padding: '40px' }}>
              Computing movement analysis...
            </p>
          )}

          {movement && movement.segments.length > 0 && (
            <div
              style={{
                background: 'var(--bg-secondary)',
                border: '1px solid var(--border-color)',
                borderRadius: 'var(--radius-lg)',
                padding: '20px',
                marginBottom: '24px',
                boxShadow: 'var(--shadow-sm)',
              }}
            >
              <h3 style={{ fontSize: '15px', fontWeight: 600, marginBottom: '16px' }}>
                Movement Segments
              </h3>
              <div style={{ overflowX: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                  <thead>
                    <tr>
                      {['From', 'To', 'Distance', 'Duration', 'Speed'].map((h) => (
                        <th
                          key={h}
                          style={{
                            padding: '10px 16px',
                            textAlign: 'left',
                            fontSize: '12px',
                            fontWeight: 600,
                            textTransform: 'uppercase',
                            letterSpacing: '0.05em',
                            color: 'var(--text-muted)',
                            borderBottom: '2px solid var(--border-color)',
                          }}
                        >
                          {h}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {movement.segments.map((seg, i) => (
                      <tr
                        key={i}
                        style={{ borderBottom: '1px solid var(--border-color)' }}
                      >
                        <td style={{ padding: '10px 16px', fontSize: '14px' }}>#{seg.from_id}</td>
                        <td style={{ padding: '10px 16px', fontSize: '14px' }}>#{seg.to_id}</td>
                        <td style={{ padding: '10px 16px', fontSize: '14px' }}>{seg.distance_km.toFixed(2)} km</td>
                        <td style={{ padding: '10px 16px', fontSize: '14px' }}>{seg.duration_hours.toFixed(1)} h</td>
                        <td style={{ padding: '10px 16px', fontSize: '14px' }}>{seg.speed_kmh.toFixed(1)} km/h</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {temporalLoading && (
            <p style={{ color: 'var(--text-muted)', textAlign: 'center', padding: '40px' }}>
              Computing temporal analysis...
            </p>
          )}

          {temporal && (
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(360px, 1fr))', gap: '16px' }}>
              <BarChart data={hourlyData} title="Hourly Distribution" color="var(--accent)" />
              <BarChart data={dailyData} title="Daily Distribution" color="var(--success)" />
              <BarChart data={monthlyData} title="Monthly Distribution" color="var(--warning)" />
            </div>
          )}

          {temporal && (
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: '1fr 1fr',
                gap: '16px',
                marginTop: '16px',
              }}
            >
              <StatsCard
                title="Peak Hour"
                value={`${String(temporal.peak_hour).padStart(2, '0')}:00`}
                icon="🕐"
              />
              <StatsCard
                title="Peak Day"
                value={['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][temporal.peak_day]}
                icon="📅"
              />
            </div>
          )}
        </>
      )}
    </div>
  )
}
