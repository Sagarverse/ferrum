import { useState, useEffect } from 'react';
import { HardDrive, CheckCircle2, AlertCircle, Plus, Trash2, Loader2 } from 'lucide-react';
import { api, formatBytes, type ReplicaRecord } from '../utils/api';

export function Storage() {
  const [repos, setRepos] = useState<ReplicaRecord[]>([]);
  const [metrics, setMetrics] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [addName, setAddName] = useState('');
  const [addPath, setAddPath] = useState('');
  const [addEncrypted, setAddEncrypted] = useState(false);
  const [adding, setAdding] = useState(false);

  const fetchData = async () => {
    try {
      const [replicaData, metricsData] = await Promise.all([
        api.getReplicas(),
        api.metrics(),
      ]);
      setRepos(replicaData.replicas);
      setMetrics(metricsData);
    } catch {}
    setLoading(false);
  };

  useEffect(() => { fetchData(); }, []);

  const addRepo = async () => {
    if (!addName.trim() || !addPath.trim()) return;
    setAdding(true);
    try {
      await api.addReplica(addName, addPath, addEncrypted);
      setShowAdd(false);
      setAddName('');
      setAddPath('');
      setAddEncrypted(false);
      await fetchData();
    } catch {}
    setAdding(false);
  };

  const totalOrig = metrics?.storage?.original_bytes ?? 0;
  const totalStored = metrics?.storage?.stored_bytes ?? 0;
  const dedup = metrics?.storage?.dedup_ratio?.toFixed(1) ?? '—';
  const saved = totalOrig > totalStored ? totalOrig - totalStored : 0;

  if (loading) {
    return <div className="empty fade-in"><Loader2 size={32} style={{ color: 'var(--mm-brand)', animation: 'mmPulse 1s infinite' }} /><p style={{ marginTop: 'var(--mm-4)' }}>Loading repositories...</p></div>;
  }

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1 className="page-title">Repositories</h1>
          <p className="page-subtitle">{repos.length} backup destinations</p>
        </div>
        <div style={{ display: 'flex', gap: 'var(--mm-2)' }}>
          <button 
            className="btn btn-secondary" 
            style={{ color: 'var(--mm-red)', borderColor: 'var(--mm-red)' }} 
            onClick={async () => {
              if (confirm('Are you sure you want to factory reset everything? This will wipe all protected items, versions, and backup data from the local repository. This action cannot be undone.')) {
                await api.resetAll();
                window.location.reload();
              }
            }}
          >
            <Trash2 size={14} /> Factory Reset
          </button>
          <button className="btn btn-primary" onClick={() => setShowAdd(!showAdd)}>
            <Plus size={14} /> Add Repository
          </button>
        </div>
      </div>

      {showAdd && (
        <div className="card fade-in" style={{ marginBottom: 'var(--mm-4)', borderColor: 'var(--mm-brand)', maxWidth: 500 }}>
          <div style={{ fontSize: 'var(--mm-text-md)', fontWeight: 600, marginBottom: 'var(--mm-3)' }}>Add Repository</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-3)' }}>
            <div>
              <label style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)', display: 'block', marginBottom: 4 }}>Name</label>
              <input className="input" placeholder="e.g. Backup SSD" value={addName} onChange={e => setAddName(e.target.value)} autoFocus />
            </div>
            <div>
              <label style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)', display: 'block', marginBottom: 4 }}>Path</label>
              <input className="input mono" placeholder={navigator.platform.includes('Win') ? 'D:\\Backups' : '/Volumes/BackupSSD'} value={addPath} onChange={e => setAddPath(e.target.value)} />
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-2)' }}>
              <input type="checkbox" id="encrypt" checked={addEncrypted} onChange={e => setAddEncrypted(e.target.checked)} style={{ accentColor: '#00B4D8' }} />
              <label htmlFor="encrypt" style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)', cursor: 'pointer' }}>Encrypt with AES-256-GCM</label>
            </div>
            <div style={{ display: 'flex', gap: 'var(--mm-2)' }}>
              <button className="btn btn-primary" onClick={addRepo} disabled={adding || !addName.trim() || !addPath.trim()}>
                {adding ? 'Adding...' : 'Add'}
              </button>
              <button className="btn btn-secondary" onClick={() => setShowAdd(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}

      {/* Metrics */}
      <div className="grid grid-4" style={{ marginBottom: 'var(--mm-6)' }}>
        <div className="metric-card">
          <div className="metric-label">Original Size</div>
          <div className="metric-value">{formatBytes(totalOrig)}</div>
          <div className="metric-sub">Total protected data</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Stored Size</div>
          <div className="metric-value" style={{ color: 'var(--mm-green)' }}>{formatBytes(totalStored)}</div>
          <div className="metric-sub">After dedup + compression</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Dedup Ratio</div>
          <div className="metric-value" style={{ color: 'var(--mm-brand)' }}>{dedup}x</div>
          <div className="metric-sub">Space efficiency</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Space Saved</div>
          <div className="metric-value" style={{ color: 'var(--mm-green)' }}>{formatBytes(saved)}</div>
          <div className="metric-sub">By deduplication</div>
        </div>
      </div>

      {repos.length === 0 ? (
        <div className="empty">
          <HardDrive size={48} />
          <h3>No repositories configured</h3>
          <p>Add an external SSD or drive as a backup repository. Ferrum will sync protected data automatically.</p>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-3)' }}>
          {repos.map(r => (
            <div key={r.id} className="card" style={{ padding: 'var(--mm-5)', opacity: r.enabled ? 1 : 0.6, filter: r.enabled ? 'none' : 'grayscale(1)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-4)' }}>
                <div className={`data-icon ${r.encrypted ? 'yellow' : 'teal'}`} style={{ width: 44, height: 44 }}>
                  <HardDrive size={22} />
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 700, fontSize: 'var(--mm-text-md)', display: 'flex', alignItems: 'center', gap: 'var(--mm-2)' }}>
                    {r.name}
                    {!r.enabled && <span className="badge">Disabled</span>}
                  </div>
                  <div style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)', fontFamily: 'var(--mm-font-mono)' }}>{r.path}</div>
                  <div style={{ display: 'flex', gap: 'var(--mm-4)', marginTop: 'var(--mm-2)', alignItems: 'center' }}>
                    <span className={`badge ${r.status === 'healthy' ? 'badge-green' : r.status === 'initializing' ? 'badge-blue' : 'badge-yellow'}`}>
                      {r.status === 'healthy' ? <CheckCircle2 size={10} /> : <AlertCircle size={10} />}
                      {r.status}
                    </span>
                    <span style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)' }}>
                      {r.encrypted ? '🔒 Encrypted (AES-256)' : '📂 Mirror'}
                    </span>
                    {r.last_sync_at && (
                      <span style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)' }}>
                        Last sync: {new Date(r.last_sync_at).toLocaleString()}
                      </span>
                    )}
                  </div>
                </div>
                <div>
                  <button 
                    className={`btn ${r.enabled ? 'btn-secondary' : 'btn-primary'}`}
                    onClick={async () => {
                      try {
                        await api.toggleReplica(r.id, !r.enabled);
                        await fetchData();
                      } catch (e: any) {
                        alert('Failed to toggle repository: ' + (e.message || 'Unknown error'));
                      }
                    }}
                  >
                    {r.enabled ? 'Disable' : 'Enable'}
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
