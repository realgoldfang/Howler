import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { sightingsApi } from '../api'
import BarChart from '../components/BarChart'
import StatsCard from '../components/StatsCard'

interface BehaviorPrediction {
  behavior: string
  confidence: number
  features: string[]
}

const BEHAVIORS = ['hunting', 'resting', 'traveling', 'playing', 'howling', 'feeding', 'denning']

function predictBehavior(sighting: { behavior?: string; count: number; timestamp: string; pack?: string }): BehaviorPrediction {
  const hour = new Date(sighting.timestamp).getHours()
  const behaviors: { behavior: string; confidence: number }[] = []

  if (hour >= 17 || hour <= 5) {
    behaviors.push({ behavior: 'hunting', confidence: 0.85 })
    behaviors.push({ behavior: 'traveling', confidence: 0.6 })
  } else if (hour >= 6 && hour <= 10) {
    behaviors.push({ behavior: 'resting', confidence: 0.7 })
    behaviors.push({ behavior: 'feeding', confidence: 0.5 })
  } else {
    behaviors.push({ behavior: 'resting', confidence: 0.6 })
    behaviors.push({ behavior: 'playing', confidence: 0.4 })
  }

  if (sighting.count > 3) {
    behaviors.push({ behavior: 'howling', confidence: 0.55 })
  }

  if (sighting.pack) {
    behaviors.push({ behavior: 'denning', confidence: 0.3 })
  }

  const sorted = behaviors.sort((a, b) => b.confidence - a.confidence)
  const top = sorted[0] || { behavior: 'unknown', confidence: 0 }

  const features: string[] = []
  if (hour >= 17 || hour <= 5) features.push('Nocturnal timing')
  if (sighting.count > 1) features.push(`Group of ${sighting.count}`)
  if (sighting.pack) features.push(`Pack: ${sighting.pack}`)
  if (hour >= 6 && hour <= 10) features.push('Dawn period')

  return {
    behavior: top.behavior,
    confidence: top.confidence,
    features,
  }
}

export default function MachineLearningPage() {
  const [selectedWolf, setSelectedWolf] = useState<string>('')

  const { data } = useQuery({
    queryKey: ['sightings', 'ml'],
    queryFn: () => sightingsApi.list({ per_page: 200 }),
  })

  const sightings = data?.items ?? []
  const wolfIds = [...new Set(sightings.filter((s) => s.wolf_id).map((s) => s.wolf_id!))]

  const filtered = selectedWolf
    ? sightings.filter((s) => s.wolf_id === selectedWolf)
    : sightings

  const predictions = useMemo(
    () => filtered.map((s) => ({ sighting: s, prediction: predictBehavior(s) })),
    [filtered],
  )

  const behaviorCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    predictions.forEach((p) => {
      counts[p.prediction.behavior] = (counts[p.prediction.behavior] || 0) + 1
    })
    return BEHAVIORS.map((b) => ({ label: b, value: counts[b] || 0 }))
  }, [predictions])

  const confidenceData = useMemo(() => {
    const buckets = [0, 0, 0, 0, 0]
    const labels = ['0-20%', '20-40%', '40-60%', '60-80%', '80-100%']
    predictions.forEach((p) => {
      const idx = Math.min(Math.floor(p.prediction.confidence * 5), 4)
      buckets[idx]++
    })
    return labels.map((l, i) => ({ label: l, value: buckets[i] }))
  }, [predictions])

  const avgConfidence = useMemo(() => {
    if (predictions.length === 0) return 0
    return predictions.reduce((sum, p) => sum + p.prediction.confidence, 0) / predictions.length
  }, [predictions])

  return (
    <div>
      <div style={{ marginBottom: '24px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px' }}>
        <div>
          <h1 style={{ fontSize: '24px', fontWeight: 700 }}>Machine Learning</h1>
          <p style={{ fontSize: '14px', color: 'var(--text-muted)', marginTop: '4px' }}>
            Behavior predictions and activity pattern analysis
          </p>
        </div>
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
      </div>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
          gap: '16px',
          marginBottom: '24px',
        }}
      >
        <StatsCard title="Sightings Analyzed" value={predictions.length} icon="🔍" />
        <StatsCard title="Avg Confidence" value={`${(avgConfidence * 100).toFixed(1)}%`} icon="🎯" />
        <StatsCard title="Behaviors Found" value={new Set(predictions.map((p) => p.prediction.behavior)).size} icon="📊" />
        <StatsCard title="Unique Wolves" value={wolfIds.length} icon="🐺" />
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(360px, 1fr))', gap: '16px', marginBottom: '24px' }}>
        <BarChart data={behaviorCounts} title="Predicted Behavior Distribution" color="var(--accent)" />
        <BarChart data={confidenceData} title="Confidence Distribution" color="var(--success)" />
      </div>

      <div
        style={{
          background: 'var(--bg-secondary)',
          border: '1px solid var(--border-color)',
          borderRadius: 'var(--radius-lg)',
          overflow: 'hidden',
          boxShadow: 'var(--shadow-sm)',
        }}
      >
        <div style={{ padding: '20px', borderBottom: '1px solid var(--border-color)' }}>
          <h3 style={{ fontSize: '15px', fontWeight: 600 }}>Behavior Predictions</h3>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                {['Sighting', 'Wolf', 'Predicted Behavior', 'Confidence', 'Key Features'].map((h) => (
                  <th
                    key={h}
                    style={{
                      padding: '12px 16px',
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
              {predictions.slice(0, 50).map(({ sighting, prediction }) => (
                <tr
                  key={sighting.id}
                  style={{ borderBottom: '1px solid var(--border-color)' }}
                >
                  <td style={{ padding: '12px 16px', fontSize: '14px' }}>#{sighting.id}</td>
                  <td style={{ padding: '12px 16px', fontSize: '14px', fontWeight: 500 }}>
                    {sighting.wolf_id || '—'}
                  </td>
                  <td style={{ padding: '12px 16px' }}>
                    <span
                      style={{
                        display: 'inline-block',
                        padding: '3px 10px',
                        borderRadius: '12px',
                        fontSize: '12px',
                        fontWeight: 600,
                        background: 'var(--accent-light)',
                        color: 'var(--accent)',
                        textTransform: 'capitalize',
                      }}
                    >
                      {prediction.behavior}
                    </span>
                  </td>
                  <td style={{ padding: '12px 16px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <div
                        style={{
                          width: '60px',
                          height: '6px',
                          background: 'var(--bg-tertiary)',
                          borderRadius: '3px',
                          overflow: 'hidden',
                        }}
                      >
                        <div
                          style={{
                            width: `${prediction.confidence * 100}%`,
                            height: '100%',
                            background: prediction.confidence > 0.7 ? 'var(--success)' : prediction.confidence > 0.4 ? 'var(--warning)' : 'var(--danger)',
                            borderRadius: '3px',
                          }}
                        />
                      </div>
                      <span style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                        {(prediction.confidence * 100).toFixed(0)}%
                      </span>
                    </div>
                  </td>
                  <td style={{ padding: '12px 16px' }}>
                    <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                      {prediction.features.map((f, i) => (
                        <span
                          key={i}
                          style={{
                            padding: '2px 8px',
                            borderRadius: '10px',
                            fontSize: '11px',
                            background: 'var(--bg-tertiary)',
                            color: 'var(--text-secondary)',
                          }}
                        >
                          {f}
                        </span>
                      ))}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
