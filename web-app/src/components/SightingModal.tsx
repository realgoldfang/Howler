import type { Sighting } from '../types'

interface SightingModalProps {
  sighting: Sighting
  onClose: () => void
}

export default function SightingModal({ sighting, onClose }: SightingModalProps) {
  return (
    <div
      onClick={onClose}
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0,0,0,0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
        padding: '20px',
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: 'var(--bg-secondary)',
          borderRadius: 'var(--radius-lg)',
          boxShadow: 'var(--shadow-lg)',
          width: '100%',
          maxWidth: '560px',
          maxHeight: '90vh',
          overflow: 'auto',
        }}
      >
        <div
          style={{
            padding: '20px 24px',
            borderBottom: '1px solid var(--border-color)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <h2 style={{ fontSize: '18px', fontWeight: 600 }}>
            Sighting #{sighting.id}
          </h2>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              fontSize: '20px',
              color: 'var(--text-muted)',
              padding: '4px',
            }}
          >
            ✕
          </button>
        </div>

        <div style={{ padding: '24px' }}>
          {sighting.photo_url && (
            <div
              style={{
                width: '100%',
                height: '200px',
                borderRadius: 'var(--radius-md)',
                overflow: 'hidden',
                marginBottom: '20px',
                background: 'var(--bg-tertiary)',
              }}
            >
              <img
                src={sighting.photo_url}
                alt="Sighting"
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              />
            </div>
          )}

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
            <Field label="Wolf ID" value={sighting.wolf_id} />
            <Field label="Pack" value={sighting.pack} />
            <Field label="Timestamp" value={new Date(sighting.timestamp).toLocaleString()} />
            <Field label="Count" value={String(sighting.count)} />
            <Field label="Behavior" value={sighting.behavior} />
            <Field label="Health" value={sighting.health} />
            <Field label="Latitude" value={sighting.latitude.toFixed(6)} />
            <Field label="Longitude" value={sighting.longitude.toFixed(6)} />
          </div>

          {sighting.notes && (
            <div style={{ marginTop: '20px' }}>
              <div
                style={{
                  fontSize: '12px',
                  fontWeight: 600,
                  textTransform: 'uppercase',
                  letterSpacing: '0.05em',
                  color: 'var(--text-muted)',
                  marginBottom: '6px',
                }}
              >
                Notes
              </div>
              <div
                style={{
                  padding: '12px',
                  background: 'var(--bg-tertiary)',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: '14px',
                  lineHeight: 1.6,
                  color: 'var(--text-secondary)',
                }}
              >
                {sighting.notes}
              </div>
            </div>
          )}

          <div
            style={{
              marginTop: '20px',
              padding: '12px',
              background: 'var(--bg-tertiary)',
              borderRadius: 'var(--radius-sm)',
              fontSize: '12px',
              color: 'var(--text-muted)',
              display: 'flex',
              justifyContent: 'space-between',
            }}
          >
            <span>Created: {new Date(sighting.created_at).toLocaleString()}</span>
            <span>Updated: {new Date(sighting.updated_at).toLocaleString()}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

function Field({ label, value }: { label: string; value?: string }) {
  return (
    <div>
      <div
        style={{
          fontSize: '12px',
          fontWeight: 600,
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          color: 'var(--text-muted)',
          marginBottom: '4px',
        }}
      >
        {label}
      </div>
      <div style={{ fontSize: '14px', color: 'var(--text-primary)', fontWeight: 500 }}>
        {value || '—'}
      </div>
    </div>
  )
}
