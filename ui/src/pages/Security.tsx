import { useState, useEffect } from 'react';
import {
  ShieldAlert, ShieldCheck, Activity, Eye, Lock,
  FileWarning, Zap, CheckCircle2, Loader2, AlertTriangle
} from 'lucide-react';
import { api, type IncidentRecord } from '../utils/api';

const DEFENSES = [
  { icon: ShieldAlert, name: 'Ransomware Detection', desc: 'Behavioral analysis + known extension monitoring' },
  { icon: Zap, name: 'Entropy Analysis', desc: 'Detects unauthorized encryption of your files' },
  { icon: Activity, name: 'Rate Monitoring', desc: 'Flags mass deletion & modification patterns' },
  { icon: Lock, name: 'Quarantine Engine', desc: 'Isolates compromised replicas automatically' },
  { icon: FileWarning, name: 'Delete Protection', desc: 'Preserves data even after source deletion' },
  { icon: Eye, name: 'Process Monitor', desc: 'Tracks which processes access protected data' },
];

export function Security() {
  const [secStatus, setSecStatus] = useState<any>(null);
  const [incidents, setIncidents] = useState<IncidentRecord[]>([]);
  const [loading, setLoading] = useState(true);
  
  // Local state for toggling defenses
  const [activeToggles, setActiveToggles] = useState<Record<string, boolean>>({
    'Ransomware Detection': true,
    'Entropy Analysis': true,
    'Rate Monitoring': true,
    'Quarantine Engine': true,
    'Delete Protection': true,
    'Process Monitor': true,
  });

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [status, incidentData] = await Promise.all([
          api.getSecurityStatus(),
          api.getIncidents(),
        ]);
        setSecStatus(status);
        setIncidents(incidentData.incidents);
      } catch {}
      setLoading(false);
    };
    fetchData();
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return <div className="empty fade-in"><Loader2 size={32} style={{ color: 'var(--mm-brand)', animation: 'mmPulse 1s infinite' }} /><p style={{ marginTop: 'var(--mm-4)' }}>Loading security status...</p></div>;
  }

  let threatLevel = secStatus?.threat_level;
  const activeIncidents = secStatus?.active_incidents ?? 0;
  const totalIncidents = secStatus?.total_incidents ?? 0;

  if (!threatLevel || threatLevel === 'unknown') {
    threatLevel = activeIncidents === 0 ? 'normal' : 'elevated';
  }

  const toggleDefense = (name: string) => {
    setActiveToggles(prev => ({
      ...prev,
      [name]: !prev[name]
    }));
  };
  
  const activeCount = Object.values(activeToggles).filter(v => v).length;

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1 className="page-title">Security Monitor</h1>
          <p className="page-subtitle">Real-time threat detection & ransomware defense</p>
        </div>
        <span className={`badge ${activeIncidents === 0 ? 'badge-green' : 'badge-red'}`} style={{ fontSize: 'var(--mm-text-sm)', padding: '4px 12px' }}>
          {activeIncidents === 0
            ? <><ShieldCheck size={14} /> All Clear</>
            : <><AlertTriangle size={14} /> {activeIncidents} Active</>
          }
        </span>
      </div>

      <div className="grid grid-4" style={{ marginBottom: 'var(--mm-6)' }}>
        <div className="metric-card">
          <div className="metric-label">Threat Level</div>
          <div className="metric-value" style={{
            color: threatLevel === 'normal' ? 'var(--mm-green)' : threatLevel === 'elevated' ? 'var(--mm-yellow)' : 'var(--mm-red)',
            textTransform: 'capitalize',
          }}>{threatLevel}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Active Incidents</div>
          <div className="metric-value" style={{ color: activeIncidents === 0 ? 'var(--mm-green)' : 'var(--mm-red)' }}>{activeIncidents}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Total Incidents</div>
          <div className="metric-value">{totalIncidents}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Defenses</div>
          <div className="metric-value" style={{ color: activeCount === DEFENSES.length ? 'var(--mm-green)' : 'var(--mm-yellow)' }}>
            {activeCount}
          </div>
          <div className="metric-sub">{activeCount === DEFENSES.length ? 'All active' : 'Some disabled'}</div>
        </div>
      </div>

      <div className="grid grid-2" style={{ gap: 'var(--mm-6)' }}>
        <div>
          <div style={{ fontSize: 'var(--mm-text-md)', fontWeight: 600, marginBottom: 'var(--mm-4)' }}>Active Defenses</div>
          <div className="card" style={{ padding: 'var(--mm-2)' }}>
            {DEFENSES.map(d => {
              const isOn = activeToggles[d.name];
              return (
                <div key={d.name} className="defense-item">
                  <div className="defense-icon" style={{ 
                    background: isOn ? 'var(--mm-green-bg)' : 'var(--mm-bg-hover)', 
                    color: isOn ? 'var(--mm-green)' : 'var(--mm-text-muted)',
                    transition: 'all 0.2s' 
                  }}>
                    <d.icon size={14} />
                  </div>
                  <div className="defense-info">
                    <div className="defense-name" style={{ color: isOn ? 'var(--mm-text)' : 'var(--mm-text-muted)' }}>{d.name}</div>
                    <div className="defense-desc">{d.desc}</div>
                  </div>
                  <button 
                    onClick={() => toggleDefense(d.name)}
                    style={{
                      background: isOn ? 'var(--mm-green)' : 'var(--mm-bg-hover)',
                      border: 'none',
                      borderRadius: '12px',
                      width: '36px',
                      height: '20px',
                      position: 'relative',
                      cursor: 'pointer',
                      transition: 'background 0.2s'
                    }}
                  >
                    <div style={{
                      position: 'absolute',
                      top: '2px',
                      left: isOn ? '18px' : '2px',
                      width: '16px',
                      height: '16px',
                      borderRadius: '50%',
                      background: isOn ? '#fff' : 'var(--mm-text-muted)',
                      transition: 'all 0.2s',
                      boxShadow: '0 1px 3px rgba(0,0,0,0.3)'
                    }} />
                  </button>
                </div>
              );
            })}
          </div>
        </div>

        <div>
          <div style={{ fontSize: 'var(--mm-text-md)', fontWeight: 600, marginBottom: 'var(--mm-4)' }}>
            Incidents {incidents.length > 0 && `(${incidents.length})`}
          </div>
          {incidents.length === 0 ? (
            <div className="card" style={{ padding: 'var(--mm-8)', textAlign: 'center' }}>
              <ShieldCheck size={32} color="var(--mm-green)" style={{ margin: '0 auto var(--mm-3)' }} />
              <div style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)' }}>
                No incidents recorded. Your data is safe.
              </div>
            </div>
          ) : (
            <div className="card" style={{ padding: 'var(--mm-2)' }}>
              {incidents.map(inc => (
                <div key={inc.id} className="defense-item">
                  <AlertTriangle size={14} color={
                    inc.severity === 'critical' ? 'var(--mm-red)' :
                    inc.severity === 'high' ? 'var(--mm-orange)' :
                    inc.severity === 'medium' ? 'var(--mm-yellow)' : 'var(--mm-text-muted)'
                  } style={{ flexShrink: 0 }} />
                  <div className="defense-info">
                    <div className="defense-name">{inc.description}</div>
                    <div className="defense-desc">
                      {inc.severity} · Score: {inc.threat_score} · {new Date(inc.created_at).toLocaleString()}
                    </div>
                  </div>
                  <span className={`badge ${inc.status === 'resolved' ? 'badge-green' : inc.status === 'active' ? 'badge-red' : 'badge-yellow'}`}>
                    {inc.status}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
