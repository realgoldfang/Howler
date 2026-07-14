import { useState, useRef } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { exportApi, importApi } from '../api'

export default function SettingsPage() {
  const queryClient = useQueryClient()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [serverUrl, setServerUrl] = useState(() => localStorage.getItem('howler-api-url') || 'http://localhost:8080')
  const [importResult, setImportResult] = useState<{ imported: number; errors: string[] } | null>(null)

  const exportMutation = useMutation({
    mutationFn: (format: 'csv' | 'geojson' | 'kml') => exportApi.download(format),
    onSuccess: (data, format) => {
      const blob = new Blob([data], {
        type: format === 'csv' ? 'text/csv' : format === 'geojson' ? 'application/json' : 'application/vnd.google-earth.kml+xml',
      })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `howler-export.${format}`
      a.click()
      URL.revokeObjectURL(url)
    },
  })

  const importMutation = useMutation({
    mutationFn: (file: File) => importApi.upload(file),
    onSuccess: (result) => {
      setImportResult(result)
      queryClient.invalidateQueries({ queryKey: ['sightings'] })
    },
  })

  const handleSaveServer = () => {
    localStorage.setItem('howler-api-url', serverUrl)
    alert('Server URL saved. Restart the app for changes to take effect.')
  }

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      importMutation.mutate(file)
    }
    e.target.value = ''
  }

  return (
    <div>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '24px', fontWeight: 700 }}>Settings</h1>
        <p style={{ fontSize: '14px', color: 'var(--text-muted)', marginTop: '4px' }}>
          Configure server connection and manage data
        </p>
      </div>

      <div style={{ display: 'grid', gap: '24px', maxWidth: '640px' }}>
        <Section title="Server Configuration">
          <div style={{ display: 'flex', gap: '12px', alignItems: 'flex-end' }}>
            <div style={{ flex: 1 }}>
              <Label>API Base URL</Label>
              <input
                type="text"
                value={serverUrl}
                onChange={(e) => setServerUrl(e.target.value)}
                placeholder="http://localhost:8080"
              />
            </div>
            <button className="primary" onClick={handleSaveServer} style={{ height: '38px' }}>
              Save
            </button>
          </div>
          <p style={{ fontSize: '13px', color: 'var(--text-muted)', marginTop: '8px' }}>
            The backend API server address. Changes require a page reload.
          </p>
        </Section>

        <Section title="Export Data">
          <p style={{ fontSize: '13px', color: 'var(--text-muted)', marginBottom: '16px' }}>
            Download all sighting data in your preferred format.
          </p>
          <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
            {(['csv', 'geojson', 'kml'] as const).map((format) => (
              <button
                key={format}
                className="secondary"
                onClick={() => exportMutation.mutate(format)}
                disabled={exportMutation.isPending}
                style={{ minWidth: '100px' }}
              >
                {exportMutation.isPending ? '...' : format.toUpperCase()}
              </button>
            ))}
          </div>
        </Section>

        <Section title="Import Data">
          <p style={{ fontSize: '13px', color: 'var(--text-muted)', marginBottom: '16px' }}>
            Import sighting data from a CSV or JSON file.
          </p>
          <input
            ref={fileInputRef}
            type="file"
            accept=".csv,.json,.geojson"
            onChange={handleImport}
            style={{ display: 'none' }}
          />
          <button
            className="primary"
            onClick={() => fileInputRef.current?.click()}
            disabled={importMutation.isPending}
          >
            {importMutation.isPending ? 'Importing...' : 'Choose File'}
          </button>

          {importMutation.isError && (
            <div
              style={{
                marginTop: '12px',
                padding: '12px',
                background: 'var(--danger-light)',
                borderRadius: 'var(--radius-sm)',
                fontSize: '13px',
                color: 'var(--danger)',
              }}
            >
              Import failed: {importMutation.error.message}
            </div>
          )}

          {importResult && (
            <div
              style={{
                marginTop: '12px',
                padding: '12px',
                background: importResult.errors.length > 0 ? 'var(--warning-light)' : 'var(--success-light)',
                borderRadius: 'var(--radius-sm)',
                fontSize: '13px',
              }}
            >
              <strong>{importResult.imported}</strong> records imported successfully.
              {importResult.errors.length > 0 && (
                <div style={{ marginTop: '8px' }}>
                  <strong>{importResult.errors.length}</strong> errors:
                  <ul style={{ margin: '4px 0 0 16px' }}>
                    {importResult.errors.slice(0, 5).map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                    {importResult.errors.length > 5 && (
                      <li>...and {importResult.errors.length - 5} more</li>
                    )}
                  </ul>
                </div>
              )}
            </div>
          )}
        </Section>

        <Section title="About">
          <div style={{ fontSize: '14px', color: 'var(--text-secondary)', lineHeight: 1.8 }}>
            <p><strong>Howler</strong> - Wolf Tracking System</p>
            <p>Version 0.1.0</p>
            <p style={{ marginTop: '8px', fontSize: '13px', color: 'var(--text-muted)' }}>
              A comprehensive wolf sighting tracking and analysis platform with
              movement analysis, temporal patterns, territory clustering, and
              behavior prediction capabilities.
            </p>
          </div>
        </Section>
      </div>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div
      style={{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)',
        borderRadius: 'var(--radius-lg)',
        padding: '24px',
        boxShadow: 'var(--shadow-sm)',
      }}
    >
      <h3 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>{title}</h3>
      {children}
    </div>
  )
}

function Label({ children }: { children: React.ReactNode }) {
  return (
    <label
      style={{
        display: 'block',
        fontSize: '13px',
        fontWeight: 500,
        color: 'var(--text-secondary)',
        marginBottom: '6px',
      }}
    >
      {children}
    </label>
  )
}
