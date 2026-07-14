interface StatsCardProps {
  title: string
  value: string | number
  icon: string
  change?: number
  subtitle?: string
}

export default function StatsCard({ title, value, icon, change, subtitle }: StatsCardProps) {
  return (
    <div
      style={{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)',
        borderRadius: 'var(--radius-lg)',
        padding: '20px',
        display: 'flex',
        alignItems: 'flex-start',
        gap: '16px',
        boxShadow: 'var(--shadow-sm)',
        transition: 'box-shadow 0.15s ease',
      }}
    >
      <div
        style={{
          width: '48px',
          height: '48px',
          borderRadius: 'var(--radius-md)',
          background: 'var(--accent-light)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '24px',
          flexShrink: 0,
        }}
      >
        {icon}
      </div>

      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontSize: '13px',
            fontWeight: 500,
            color: 'var(--text-muted)',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
          }}
        >
          {title}
        </div>
        <div
          style={{
            fontSize: '28px',
            fontWeight: 700,
            color: 'var(--text-primary)',
            lineHeight: 1.2,
            marginTop: '4px',
          }}
        >
          {value}
        </div>
        {(change !== undefined || subtitle) && (
          <div style={{ marginTop: '6px', display: 'flex', alignItems: 'center', gap: '8px' }}>
            {change !== undefined && (
              <span
                style={{
                  fontSize: '13px',
                  fontWeight: 600,
                  color: change >= 0 ? 'var(--success)' : 'var(--danger)',
                }}
              >
                {change >= 0 ? '↑' : '↓'} {Math.abs(change)}%
              </span>
            )}
            {subtitle && (
              <span style={{ fontSize: '13px', color: 'var(--text-muted)' }}>
                {subtitle}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
