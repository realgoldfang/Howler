import { useState, useEffect } from 'react'
import { Outlet, NavLink, useLocation } from 'react-router-dom'
import type { Theme } from '../types'

const navItems = [
  { to: '/', label: 'Dashboard', icon: '📊' },
  { to: '/map', label: 'Map', icon: '🗺️' },
  { to: '/analysis', label: 'Analysis', icon: '📈' },
  { to: '/ml', label: 'Machine Learning', icon: '🧠' },
  { to: '/settings', label: 'Settings', icon: '⚙️' },
]

export default function Layout() {
  const [theme, setTheme] = useState<Theme>(() => {
    return (localStorage.getItem('howler-theme') as Theme) || 'light'
  })
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const location = useLocation()

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
    localStorage.setItem('howler-theme', theme)
  }, [theme])

  useEffect(() => {
    setSidebarOpen(true)
  }, [location])

  return (
    <div style={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <aside
        style={{
          width: sidebarOpen ? 'var(--sidebar-width)' : '0',
          minWidth: sidebarOpen ? 'var(--sidebar-width)' : '0',
          background: 'var(--bg-secondary)',
          borderRight: '1px solid var(--border-color)',
          display: 'flex',
          flexDirection: 'column',
          transition: 'width 0.2s ease, min-width 0.2s ease',
          overflow: 'hidden',
        }}
      >
        <div
          style={{
            padding: '16px 20px',
            borderBottom: '1px solid var(--border-color)',
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
          }}
        >
          <span style={{ fontSize: '24px' }}>🐺</span>
          <span
            style={{
              fontSize: '18px',
              fontWeight: 700,
              color: 'var(--text-primary)',
              whiteSpace: 'nowrap',
            }}
          >
            Howler
          </span>
        </div>

        <nav style={{ padding: '12px 8px', flex: 1, overflowY: 'auto' }}>
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              style={({ isActive }) => ({
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                padding: '10px 12px',
                borderRadius: 'var(--radius-sm)',
                color: isActive ? 'var(--accent)' : 'var(--text-secondary)',
                background: isActive ? 'var(--accent-light)' : 'transparent',
                fontWeight: isActive ? 600 : 400,
                fontSize: '14px',
                transition: 'all 0.15s ease',
                whiteSpace: 'nowrap',
                textDecoration: 'none',
              })}
            >
              <span style={{ fontSize: '18px' }}>{item.icon}</span>
              {item.label}
            </NavLink>
          ))}
        </nav>

        <div
          style={{
            padding: '12px 16px',
            borderTop: '1px solid var(--border-color)',
            fontSize: '12px',
            color: 'var(--text-muted)',
            whiteSpace: 'nowrap',
          }}
        >
          Wolf Tracking System v0.1.0
        </div>
      </aside>

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <header
          style={{
            height: 'var(--header-height)',
            borderBottom: '1px solid var(--border-color)',
            background: 'var(--bg-secondary)',
            display: 'flex',
            alignItems: 'center',
            padding: '0 24px',
            gap: '16px',
          }}
        >
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            style={{
              background: 'none',
              padding: '6px',
              fontSize: '20px',
              color: 'var(--text-secondary)',
            }}
          >
            ☰
          </button>

          <div style={{ flex: 1 }} />

          <button
            onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
            className="secondary"
            style={{ fontSize: '16px', padding: '6px 12px' }}
          >
            {theme === 'light' ? '🌙' : '☀️'}
          </button>
        </header>

        <main
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: '24px',
          }}
        >
          <Outlet />
        </main>
      </div>
    </div>
  )
}
