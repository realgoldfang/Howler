interface BarChartProps {
  data: { label: string; value: number }[]
  title?: string
  maxValue?: number
  color?: string
  height?: number
}

export default function BarChart({
  data,
  title,
  maxValue,
  color = 'var(--accent)',
  height = 200,
}: BarChartProps) {
  const max = maxValue || Math.max(...data.map((d) => d.value), 1)

  return (
    <div
      style={{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)',
        borderRadius: 'var(--radius-lg)',
        padding: '20px',
        boxShadow: 'var(--shadow-sm)',
      }}
    >
      {title && (
        <h3
          style={{
            fontSize: '15px',
            fontWeight: 600,
            marginBottom: '16px',
            color: 'var(--text-primary)',
          }}
        >
          {title}
        </h3>
      )}

      <div
        style={{
          display: 'flex',
          alignItems: 'flex-end',
          gap: '2px',
          height: `${height}px`,
          paddingBottom: '24px',
          position: 'relative',
        }}
      >
        {[0.25, 0.5, 0.75, 1].map((pct) => (
          <div
            key={pct}
            style={{
              position: 'absolute',
              left: 0,
              right: 0,
              bottom: `${24 + pct * (height - 24)}px`,
              height: '1px',
              background: 'var(--border-color)',
              zIndex: 0,
            }}
          />
        ))}

        {data.map((d, i) => {
          const h = max > 0 ? (d.value / max) * (height - 32) : 0
          return (
            <div
              key={i}
              style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                position: 'relative',
                zIndex: 1,
              }}
            >
              <div
                style={{
                  fontSize: '11px',
                  color: 'var(--text-muted)',
                  marginBottom: '4px',
                  fontWeight: 500,
                }}
              >
                {d.value}
              </div>
              <div
                style={{
                  width: '100%',
                  maxWidth: '40px',
                  height: `${h}px`,
                  background: color,
                  borderRadius: '4px 4px 0 0',
                  transition: 'height 0.3s ease',
                  minHeight: d.value > 0 ? '2px' : '0',
                }}
              />
              <div
                style={{
                  fontSize: '11px',
                  color: 'var(--text-muted)',
                  marginTop: '6px',
                  textAlign: 'center',
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  width: '100%',
                }}
                title={d.label}
              >
                {d.label}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
