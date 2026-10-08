import { useState, useEffect } from 'react';
import { Clock, RotateCcw, Download, CheckCircle2, FileText, FolderOpen, Loader2, AlertTriangle, Play, Layers, ArrowUpRight, ArrowDownRight, Trash2 } from 'lucide-react';
import { api, formatBytes, timeAgo, formatDate, type ProtectedItem, type VersionRecord } from '../utils/api';

export function Versions() {
  const [items, setItems] = useState<ProtectedItem[]>([]);
  const [selectedItem, setSelectedItem] = useState<string | null>(null);
  const [versions, setVersions] = useState<VersionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [versionsLoading, setVersionsLoading] = useState(false);
  const [restoring, setRestoring] = useState<string | null>(null);
  const [restoredVersion, setRestoredVersion] = useState<string | null>(null);
  const [restoreCopies, setRestoreCopies] = useState<number>(1);
  const [backingUp, setBackingUp] = useState(false);
  const [backupSuccess, setBackupSuccess] = useState(false);

  useEffect(() => {
    const fetchItems = async () => {
      try {
        const data = await api.getItems();
        setItems(data.items);
        if (data.items.length > 0 && !selectedItem) {
          setSelectedItem(data.items[0].id);
        }
      } catch {}
      setLoading(false);
    };
    fetchItems();
  }, []);

  const fetchVersions = async () => {
    if (!selectedItem) return;
    setVersionsLoading(true);
    try {
      const data = await api.getVersions(selectedItem);
      setVersions(data.versions ?? []);
    } catch {
      setVersions([]);
    }
    setVersionsLoading(false);
  };

  useEffect(() => {
    fetchVersions();
  }, [selectedItem]);

  // FIX: Pass version_number (as string) instead of version UUID id
  const handleRestore = async (versionNumber: number) => {
    if (!selectedItem) return;
    
    let dest = navigator.platform.includes('Win') ? 'C:\\Restored' : '/Users/sagarm/Restored';
    
    // Use native Electron folder picker instead of blocked window.prompt()
    if (window.metallic?.selectFolder) {
      const selected = await window.metallic.selectFolder();
      if (!selected) return; // User cancelled
      dest = selected;
    } else {
      // Fallback for non-electron env
      const p = prompt('Restore destination path:', dest);
      if (!p) return;
      dest = p;
    }

    setRestoring(String(versionNumber));
    try {
      for (let i = 0; i < restoreCopies; i++) {
        const finalDest = restoreCopies > 1 ? `${dest}-copy-${i + 1}` : dest;
        await api.restore(selectedItem, String(versionNumber), finalDest);
      }
      setRestoredVersion(String(versionNumber));
      setTimeout(() => setRestoredVersion(null), 4000);
    } catch (e: any) {
      alert('Restore failed: ' + (e.message || 'Unknown error'));
    }
    setRestoring(null);
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this version?')) {
      try {
        await api.deleteVersion(id);
        setVersions(prev => prev.filter(v => v.id !== id));
      } catch (e: any) {
        alert('Failed to delete version: ' + (e.message || 'Unknown error'));
      }
    }
  };

  const handleBackup = async () => {
    if (!selectedItem) return;
    setBackingUp(true);
    try {
      await api.triggerBackup(selectedItem);
      setBackupSuccess(true);
      setTimeout(() => setBackupSuccess(false), 3000);
      await fetchVersions();
    } catch (e: any) {
      alert('Backup failed: ' + (e.message || 'Unknown error'));
    }
    setBackingUp(false);
  };

  // Sort versions newest first
  const sorted = [...versions].sort((a, b) => b.version_number - a.version_number);

  if (loading) {
    return (
      <div className="empty fade-in">
        <Loader2 size={32} className="animate-spin" style={{ color: 'var(--mm-brand)' }} />
        <p style={{ marginTop: 'var(--mm-4)' }}>Loading...</p>
      </div>
    );
  }

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1 className="page-title">Version Timeline</h1>
          <p className="page-subtitle">
            {versions.length} snapshots
            {selectedItem && items.length > 0 && ` · ${items.find(i => i.id === selectedItem)?.path.split(/[/\\]/).pop() || ''}`}
          </p>
        </div>
        <div style={{ display: 'flex', gap: 'var(--mm-2)', alignItems: 'center' }}>
          {items.length > 1 && (
            <select
              className="select"
              value={selectedItem || ''}
              onChange={e => setSelectedItem(e.target.value)}
            >
              {items.map(i => (
                <option key={i.id} value={i.id}>{i.path.split(/[/\\]/).pop()}</option>
              ))}
            </select>
          )}
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-2)' }}>
            <span style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-muted)' }}>Restore Copies:</span>
            <input 
              type="number" 
              className="input" 
              style={{ width: 60, padding: '4px 8px', minHeight: 32 }}
              min={1} 
              max={10} 
              value={restoreCopies} 
              onChange={e => setRestoreCopies(parseInt(e.target.value) || 1)}
            />
          </div>
          <button
            className="btn btn-primary"
            onClick={handleBackup}
            disabled={backingUp || !selectedItem}
          >
            {backingUp ? <Loader2 size={14} className="animate-spin" /> : backupSuccess ? <CheckCircle2 size={14} /> : <Play size={14} />}
            {backingUp ? 'Creating…' : backupSuccess ? 'Created!' : 'New Snapshot'}
          </button>
        </div>
      </div>

      {/* Success banner */}
      {restoredVersion && (
        <div className="card fade-in" style={{
          borderColor: 'var(--mm-green)',
          background: 'var(--mm-green-bg)',
          marginBottom: 'var(--mm-4)',
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--mm-3)',
        }}>
          <CheckCircle2 size={18} color="var(--mm-green)" />
          <span style={{ color: 'var(--mm-green)', fontWeight: 600 }}>
            Version {restoredVersion} restored successfully
          </span>
        </div>
      )}

      {items.length === 0 ? (
        <div className="empty">
          <Clock size={48} />
          <h3>No protected items</h3>
          <p>Add a folder or file to protection first. Versions will appear here as backups are created.</p>
        </div>
      ) : versionsLoading ? (
        <div className="empty">
          <Loader2 size={32} className="animate-spin" style={{ color: 'var(--mm-brand)' }} />
          <p style={{ marginTop: 'var(--mm-4)' }}>Loading versions...</p>
        </div>
      ) : versions.length === 0 ? (
        <div className="empty">
          <Clock size={48} />
          <h3>No versions yet</h3>
          <p>Click "New Snapshot" above to create your first backup, or versions will be created automatically when file changes are detected.</p>
        </div>
      ) : (
        <div className="timeline">
          {sorted.map((v, idx) => {
            const prev = sorted[idx + 1];
            const sizeDelta = prev ? v.total_size - prev.total_size : v.total_size;
            const isGrowing = sizeDelta >= 0;
            const isEmpty = v.total_size === 0;

            return (
              <div key={v.id} className="tl-item fade-in">
                <div className={`tl-dot ${isEmpty ? 'yellow' : v.status === 'healthy' ? 'green' : v.status === 'suspicious' ? 'yellow' : 'red'}`} />
                <div className="card version-card">
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ flex: 1 }}>
                      <div className="tl-time">{formatDate(v.created_at)} · {timeAgo(v.created_at)}</div>
                      <div className="tl-title">
                        Version {v.version_number}
                        {idx === 0 && <span className="badge badge-blue" style={{ marginLeft: 8 }}>latest</span>}
                        {idx === sorted.length - 1 && <span className="badge" style={{ marginLeft: 8 }}>initial</span>}
                        {isEmpty && <span className="badge badge-yellow" style={{ marginLeft: 8 }}>empty</span>}
                      </div>
                      <div className="tl-desc">
                        <span><Layers size={11} style={{ verticalAlign: -1 }} /> {v.total_chunks} chunks</span>
                        <span>{formatBytes(v.total_size)}</span>
                        {prev && (
                          <span style={{
                            color: isGrowing ? 'var(--mm-green)' : 'var(--mm-yellow)',
                            display: 'flex',
                            alignItems: 'center',
                            gap: 2,
                          }}>
                            {isGrowing ? <ArrowUpRight size={11} /> : <ArrowDownRight size={11} />}
                            {isGrowing ? '+' : ''}{formatBytes(Math.abs(sizeDelta))}
                          </span>
                        )}
                      </div>
                    </div>
                    <div style={{ display: 'flex', gap: 'var(--mm-2)', alignItems: 'center' }}>
                      <span className={`badge ${v.status === 'healthy' ? 'badge-green' : 'badge-yellow'}`}>
                        {v.status === 'healthy' ? <CheckCircle2 size={10} /> : <AlertTriangle size={10} />}
                        {v.status}
                      </span>
                      <button
                        className="btn btn-secondary"
                        style={{ padding: '4px 12px' }}
                        onClick={() => handleRestore(v.version_number)}
                        disabled={restoring === String(v.version_number) || isEmpty}
                        title={isEmpty ? 'Cannot restore empty version' : `Restore version ${v.version_number}`}
                      >
                        {restoring === String(v.version_number) ? (
                          <Loader2 size={12} className="animate-spin" />
                        ) : (
                          <RotateCcw size={12} />
                        )}
                        <span style={{ fontSize: 'var(--mm-text-xs)' }}>
                          {restoring === String(v.version_number) ? 'Restoring…' : 'Restore'}
                        </span>
                      </button>
                      <button 
                        className="btn btn-secondary" 
                        style={{ padding: '4px 8px', color: 'var(--mm-red)' }}
                        onClick={() => handleDelete(v.id)}
                        title="Delete Version"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
