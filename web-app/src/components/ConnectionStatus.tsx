interface ConnectionStatusProps {
  isConnected: boolean
}

export default function ConnectionStatus({ isConnected }: ConnectionStatusProps) {
  return (
    <div
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '6px',
        padding: '4px 8px',
        borderRadius: '4px',
        backgroundColor: isConnected ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)',
        fontSize: '12px',
        color: isConnected ? 'rgb(34, 197, 94)' : 'rgb(239, 68, 68)',
        fontWeight: 500,
      }}
      title={isConnected ? 'Connected to real-time updates' : 'Disconnected from real-time updates'}
    >
      <div
        style={{
          width: '8px',
          height: '8px',
          borderRadius: '50%',
          backgroundColor: isConnected ? 'rgb(34, 197, 94)' : 'rgb(239, 68, 68)',
        }}
      />
      {isConnected ? 'Connected' : 'Disconnected'}
    </div>
  )
}
