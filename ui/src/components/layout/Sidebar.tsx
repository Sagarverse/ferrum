import { useState, useEffect } from 'react';
import type { PageId } from '../../App';
import { Logo } from '../Logo';
import {
  LayoutDashboard, Shield, HardDrive, Clock, RotateCcw, Database,
  ShieldAlert, AlertTriangle, Calendar, Cloud, BarChart3,
  Settings as SettingsIcon, Activity, Wifi, WifiOff, Sparkles
} from 'lucide-react';
import { api } from '../../utils/api';

interface SidebarProps {
  currentPage: PageId;
  onNavigate: (page: PageId) => void;
}

const NAV = [
  {
    label: 'Overview',
    items: [
      { id: 'home' as PageId, label: 'Dashboard', icon: LayoutDashboard },
      { id: 'flow' as PageId, label: 'How It Works', icon: Activity },
    ],
  },
  {
    label: 'Protection',
    items: [
      { id: 'protected' as PageId, label: 'Protected Data', icon: Shield },
      { id: 'versions' as PageId, label: 'Versions', icon: Clock },
      { id: 'recovery' as PageId, label: 'Recovery', icon: RotateCcw },
    ],
  },
  {
    label: 'Infrastructure',
    items: [
      { id: 'storage' as PageId, label: 'Repositories', icon: Database },
      { id: 'security' as PageId, label: 'Security', icon: ShieldAlert },
    ],
  },
  {
    label: 'System',
    items: [
      { id: 'settings' as PageId, label: 'Settings', icon: SettingsIcon },
      { id: 'assistant' as PageId, label: 'Assistant', icon: Sparkles },
    ],
  },
];

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  const [online, setOnline] = useState(false);
  const [threatLevel, setThreatLevel] = useState('unknown');
  const [activeIncidents, setActiveIncidents] = useState(0);
  const [protectedCount, setProtectedCount] = useState(0);

  useEffect(() => {
    const poll = async () => {
      try {
        const status = await api.status();
        setOnline(true);
        setThreatLevel(status.security?.threat_level ?? 'unknown');
        setActiveIncidents(status.security?.active_incidents ?? 0);
        setProtectedCount(status.protected_items ?? 0);
      } catch {
        setOnline(false);
      }
    };
    poll();
    const id = setInterval(poll, 5000);
    return () => clearInterval(id);
  }, []);

  const statusColor = !online
    ? 'red'
    : activeIncidents > 0
      ? 'red'
      : threatLevel === 'elevated'
        ? 'yellow'
        : 'green';

  const statusText = !online
    ? 'Engine Offline'
    : activeIncidents > 0
      ? `${activeIncidents} Active Threat${activeIncidents > 1 ? 's' : ''}`
      : 'All Systems Normal';

  const statusSub = !online
    ? 'Starting up…'
    : `${protectedCount} items protected`;

  return (
    <aside className="sidebar">
      <div className="sidebar-drag-strip" />

      <nav className="sidebar-nav">
        {NAV.map((section) => (
          <div key={section.label} className="nav-section">
            <div className="nav-section-label">{section.label}</div>
            {section.items.map((item) => (
              <button
                key={item.id}
                className={`nav-item${currentPage === item.id ? ' active' : ''}`}
                onClick={() => onNavigate(item.id)}
              >
                <item.icon />
                <span>{item.label}</span>
                {item.id === 'security' && activeIncidents > 0 && (
                  <span className="nav-badge">{activeIncidents}</span>
                )}
              </button>
            ))}
          </div>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="sidebar-brand-row">
          <div className="sidebar-logo">
            <Logo size={28} />
          </div>
          <div className="sidebar-brand">
            <h1>Metallic Mini</h1>
            <span>Data Protection</span>
          </div>
        </div>
        <div className="sidebar-status" onClick={() => onNavigate('security')}>
          <span className={`status-dot ${statusColor}`} />
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: 'var(--mm-text-sm)', fontWeight: 600, color: 'var(--mm-text)' }}>
              {statusText}
            </div>
            <div style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)' }}>
              {statusSub}
            </div>
          </div>
        </div>
      </div>
    </aside>
  );
}
