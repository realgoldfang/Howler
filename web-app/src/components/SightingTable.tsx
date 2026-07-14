import { useMemo, useState } from 'react'
import type { Sighting } from '../types'

interface SightingTableProps {
  sightings: Sighting[]
  onRowClick?: (sighting: Sighting) => void
  loading?: boolean
}

type SortKey = 'id' | 'timestamp' | 'wolf_id' | 'pack' | 'behavior' | 'count'
type SortDir = 'asc' | 'desc'

export default function SightingTable({ sightings, onRowClick, loading }: SightingTableProps) {
  const [search, setSearch] = useState('')
  const [sortKey, setSortKey] = useState<SortKey>('timestamp')
  const [sortDir, setSortDir] = useState<SortDir>('desc')
  const [page, setPage] = useState(1)
  const perPage = 10

  const filtered = useMemo(() => {
    if (!search) return sightings
    const q = search.toLowerCase()
    return sightings.filter(
      (s) =>
        (s.wolf_id && s.wolf_id.toLowerCase().includes(q)) ||
        (s.pack && s.pack.toLowerCase().includes(q)) ||
        (s.behavior && s.behavior.toLowerCase().includes(q)) ||
        (s.notes && s.notes.toLowerCase().includes(q)),
    )
  }, [sightings, search])

  const sorted = useMemo(() => {
    return [...filtered].sort((a, b) => {
      const aVal = a[sortKey] ?? ''
      const bVal = b[sortKey] ?? ''
      const cmp = typeof aVal === 'number' && typeof bVal === 'number'
        ? aVal - bVal
        : String(aVal).localeCompare(String(bVal))
      return sortDir === 'asc' ? cmp : -cmp
    })
  }, [filtered, sortKey, sortDir])

  const totalPages = Math.ceil(sorted.length / perPage)
  const paged = sorted.slice((page - 1) * perPage, page * perPage)

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(sortDir === 'asc' ? 'desc' : 'asc')
    } else {
      setSortKey(key)
      setSortDir('asc')
    }
  }

  const thStyle = (key: SortKey): React.CSSProperties => ({
    padding: '12px 16px',
    textAlign: 'left',
    fontSize: '12px',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
    borderBottom: '2px solid var(--border-color)',
    cursor: 'pointer',
    userSelect: 'none',
    whiteSpace: 'nowrap',
    color: sortKey === key ? 'var(--accent)' : 'var(--text-muted)',
  })

  return (
    <div
      style={{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)',
        borderRadius: 'var(--radius-lg)',
        overflow: 'hidden',
        boxShadow: 'var(--shadow-sm)',
      }}
    >
      <div
        style={{
          padding: '16px 20px',
          borderBottom: '1px solid var(--border-color)',
          display: 'flex',
          alignItems: 'center',
          gap: '12px',
        }}
      >
        <input
          type="text"
          placeholder="Search sightings..."
          value={search}
          onChange={(e) => { setSearch(e.target.value); setPage(1) }}
          style={{ maxWidth: '300px' }}
        />
        <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>
          {filtered.length} result{filtered.length !== 1 ? 's' : ''}
        </span>
      </div>

      <div style={{ overflowX: 'auto' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr>
              <th style={thStyle('id')} onClick={() => toggleSort('id')}>
                ID {sortKey === 'id' && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
              <th style={thStyle('timestamp')} onClick={() => toggleSort('timestamp')}>
                Time {sortKey === 'timestamp' && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
              <th style={thStyle('wolf_id')} onClick={() => toggleSort('wolf_id')}>
                Wolf {sortKey === 'wolf_id' && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
              <th style={thStyle('pack')} onClick={() => toggleSort('pack')}>
                Pack {sortKey === 'pack' && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
              <th style={thStyle('behavior')} onClick={() => toggleSort('behavior')}>
                Behavior {sortKey === 'behavior' && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
              <th style={thStyle('count')} onClick={() => toggleSort('count')}>
                Count {sortKey === 'count' && (sortDir === 'asc' ? '↑' : '↓')}
              </th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={6} style={{ padding: '40px', textAlign: 'center', color: 'var(--text-muted)' }}>
                  Loading...
                </td>
              </tr>
            ) : paged.length === 0 ? (
              <tr>
                <td colSpan={6} style={{ padding: '40px', textAlign: 'center', color: 'var(--text-muted)' }}>
                  No sightings found
                </td>
              </tr>
            ) : (
              paged.map((s) => (
                <tr
                  key={s.id}
                  onClick={() => onRowClick?.(s)}
                  style={{
                    cursor: onRowClick ? 'pointer' : 'default',
                    borderBottom: '1px solid var(--border-color)',
                    transition: 'background 0.1s ease',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--bg-hover)')}
                  onMouseLeave={(e) => (e.currentTarget.style.background = '')}
                >
                  <td style={{ padding: '12px 16px', fontSize: '14px' }}>#{s.id}</td>
                  <td style={{ padding: '12px 16px', fontSize: '14px', whiteSpace: 'nowrap' }}>
                    {new Date(s.timestamp).toLocaleString()}
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '14px', fontWeight: 500 }}>
                    {s.wolf_id || '—'}
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '14px' }}>{s.pack || '—'}</td>
                  <td style={{ padding: '12px 16px', fontSize: '14px' }}>
                    {s.behavior ? (
                      <span
                        style={{
                          display: 'inline-block',
                          padding: '2px 8px',
                          borderRadius: '12px',
                          fontSize: '12px',
                          fontWeight: 500,
                          background: 'var(--accent-light)',
                          color: 'var(--accent)',
                        }}
                      >
                        {s.behavior}
                      </span>
                    ) : '—'}
                  </td>
                  <td style={{ padding: '12px 16px', fontSize: '14px', textAlign: 'center' }}>
                    {s.count}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div
          style={{
            padding: '12px 20px',
            borderTop: '1px solid var(--border-color)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>
            Page {page} of {totalPages}
          </span>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              className="secondary"
              disabled={page <= 1}
              onClick={() => setPage(page - 1)}
              style={{ padding: '4px 12px', fontSize: '13px' }}
            >
              Prev
            </button>
            <button
              className="secondary"
              disabled={page >= totalPages}
              onClick={() => setPage(page + 1)}
              style={{ padding: '4px 12px', fontSize: '13px' }}
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
