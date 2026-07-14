import { Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import DashboardPage from './pages/DashboardPage'
import MapPage from './pages/MapPage'
import AnalysisPage from './pages/AnalysisPage'
import MachineLearningPage from './pages/MachineLearningPage'
import SettingsPage from './pages/SettingsPage'

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/map" element={<MapPage />} />
        <Route path="/analysis" element={<AnalysisPage />} />
        <Route path="/ml" element={<MachineLearningPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  )
}
