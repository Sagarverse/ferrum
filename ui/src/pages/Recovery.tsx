import { useState, useEffect, useMemo } from 'react';
import {
  RotateCcw, Clock, Download, CheckCircle2, AlertTriangle, X, Loader2,
  FileText, HardDrive, Inbox, FolderOpen, Shield
} from 'lucide-react';
import { api, formatBytes, formatDate, timeAgo, type ProtectedItem, type VersionRecord } from '../utils/api';

export function Recovery() {
  const [items, setItems] = useState<ProtectedItem[]>([]);
  const [selectedItem, setSelectedItem] = useState<string>('');
  const [versions, setVersions] = useState<VersionRecord[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(false);
  const [dest, setDest] = useState('');
  const [restoring, setRestoring] = useState(false);
  const [result, setResult] = useState<{ version: string; destination: string; files: number } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedVersion, setSelectedVersion] = useState<string>('');

  // Initial load
  useEffect(() => {
    api.getItems().then(d => {
      setItems(d.items);
      if (d.items.length > 0) {
        setSelectedItem(d.items[0].id);
      }
      const base = navigator.platform.includes('Win') ? 'C:\\Restored\\' : '/Users/sagarm/Restored/';
      setDest(base);
    }).catch(() => {});
  }, []);

  // Fetch versions whenever the selected item changes
  useEffect(() => {
    if (!selectedItem) { setVersions([]); return; }
    setLoadingVersions(true);
    setSelectedVersion('');
    setResult(null);
    setError(null);
    api.getVersions(selectedItem)
      .then(d => {
        const v = d.versions ?? [];
        setVersions(v);
        // Auto-select the latest non-empty version
        const sorted = [...v].sort((a, b) => b.version_number - a.version_number);
        const best = sorted.find(v => v.total_size > 0) || sorted[0];
        if (best) setSelectedVersion(String(best.version_number));
      })
      .catch(() => setVersions([]))
      .finally(() => setLoadingVersions(false));
  }, [selectedItem]);

  const sortedVersions = useMemo(
    () => [...versions].sort((a, b) => b.version_number - a.version_number),
    [versions]
  );

  const canRestore = !!selectedItem && !!selectedVersion && !!dest.trim();

  const startRestore = async () => {
    if (!canRestore) return;
    setRestoring(true);
    setError(null);
    setResult(null);
    try {
      const res = await api.restore(selectedItem, selectedVersion, dest);
      if (res?.errors && res.errors.length > 0) {
        setError(`Restore warning: ${res.errors.join('; ')}`);
      }
      setResult({
        version: selectedVersion,
        destination: res?.destination || dest,
        files: res?.files_restored ?? 0,
      });
    } catch (e: any) {
      setError(e.message || 'Restore failed.');
    }
    setRestoring(false);
  };

  const browseDest = async () => {
    const bridge = window.ferrum || window.metallic;
    if (bridge?.selectFolder) {
      const folder = await bridge.selectFolder();
      if (folder) setDest(folder);
    }
  };

  if (items.length === 0) {
    return (
      <div className="fade-in">
        <div className="page-header">
          <h1 className="page-title">Recovery Center</h1>
          <p className="page-subtitle">Restore data from any point in time</p>
        </div>
        <div className="empty">
          <RotateCcw size={48} />
          <h3>Nothing to recover</h3>
          <p>Add items to protection first. Recovery options will appear here.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1 className="page-title">Recovery Center</h1>
          <p className="page-subtitle">Restore your data from any version — pick, preview, recover.</p>
        </div>
      </div>

      {/* Step 1: Pick item */}
      <div className="recovery-steps">
        <div className="card recovery-step" style={{ marginBottom: 'var(--mm-4)' }}>
          <div className="recovery-step-header">
            <div className="recovery-step-number">1</div>
            <div>
              <div className="recovery-step-title">Select Protected Item</div>
              <div className="recovery-step-desc">Choose which folder or file to restore</div>
            </div>
          </div>
          <select
            className="select"
            value={selectedItem}
            onChange={(e) => setSelectedItem(e.target.value)}
            style={{ width: '100%', marginTop: 'var(--mm-3)' }}
          >
            {items.map((i) => (
              <option key={i.id} value={i.id}>{i.path}</option>
            ))}
          </select>
        </div>

        {/* Step 2: Pick version */}
        <div className="card recovery-step" style={{ marginBottom: 'var(--mm-4)' }}>
          <div className="recovery-step-header">
            <div className="recovery-step-number">2</div>
            <div>
              <div className="recovery-step-title">Choose Version</div>
              <div className="recovery-step-desc">
                {sortedVersions.length} version{sortedVersions.length !== 1 ? 's' : ''} available
              </div>
            </div>
          </div>

          {loadingVersions ? (
            <div style={{ padding: 'var(--mm-4)', textAlign: 'center' }}>
              <Loader2 size={20} className="animate-spin" style={{ color: 'var(--mm-brand)' }} />
            </div>
          ) : sortedVersions.length === 0 ? (
            <div style={{ padding: 'var(--mm-4)', textAlign: 'center', color: 'var(--mm-text-muted)' }}>
              No versions yet — protect a folder and create a backup first.
            </div>
          ) : (
            <div style={{ marginTop: 'var(--mm-3)', maxHeight: 300, overflowY: 'auto' }}>
              {sortedVersions.map((v, idx) => {
                const isEmpty = v.total_size === 0;
                const isSelected = selectedVersion === String(v.version_number);
                return (
                  <button
                    key={v.id}
                    className="version-pick-row"
                    onClick={() => { setSelectedVersion(String(v.version_number)); setResult(null); setError(null); }}
                    style={{
                      borderColor: isSelected ? 'var(--mm-brand)' : 'transparent',
                      background: isSelected ? 'var(--mm-brand-light)' : 'transparent',
                      opacity: isEmpty ? 0.5 : 1,
                    }}
                    title={isEmpty ? 'Empty version — source may have been missing' : ''}
                  >
                    <div style={{
                      width: 28, height: 28, borderRadius: 6,
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      background: isEmpty ? 'var(--mm-bg-input)' : isSelected ? 'var(--mm-brand-light)' : 'var(--mm-green-bg)',
                      color: isEmpty ? 'var(--mm-text-muted)' : isSelected ? 'var(--mm-brand)' : 'var(--mm-green)',
                      flexShrink: 0,
                    }}>
                      {isEmpty ? <Inbox size={14} /> : isSelected ? <CheckCircle2 size={14} /> : <FileText size={14} />}
                    </div>
                    <div style={{ flex: 1, textAlign: 'left' }}>
                      <div style={{ fontWeight: 600, fontSize: 'var(--mm-text-sm)', color: isEmpty ? 'var(--mm-text-muted)' : 'var(--mm-text)' }}>
                        Version {v.version_number}
                        {idx === 0 && <span className="badge badge-blue" style={{ marginLeft: 6 }}>latest</span>}
                        {idx === sortedVersions.length - 1 && <span className="badge" style={{ marginLeft: 6 }}>initial</span>}
                        {isEmpty && <span className="badge badge-yellow" style={{ marginLeft: 6 }}>empty</span>}
                      </div>
                      <div style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)' }}>
                        {formatDate(v.created_at)} · {timeAgo(v.created_at)}
                      </div>
                    </div>
                    <div style={{
                      fontFamily: 'var(--mm-font-mono)', fontSize: 'var(--mm-text-xs)',
                      color: isEmpty ? 'var(--mm-text-muted)' : 'var(--mm-text-secondary)',
                      whiteSpace: 'nowrap',
                    }}>
                      {v.total_chunks} chunks · {formatBytes(v.total_size)}
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* Step 3: Choose destination & restore */}
        <div className="card recovery-step" style={{ marginBottom: 'var(--mm-4)' }}>
          <div className="recovery-step-header">
            <div className="recovery-step-number">3</div>
            <div>
              <div className="recovery-step-title">Restore Destination</div>
              <div className="recovery-step-desc">Where files will be written</div>
            </div>
          </div>
          <div style={{ display: 'flex', gap: 'var(--mm-2)', marginTop: 'var(--mm-3)' }}>
            <input
              className="input mono"
              value={dest}
              onChange={(e) => setDest(e.target.value)}
              style={{ flex: 1 }}
              placeholder="/path/to/restore"
            />
            {(window.ferrum || window.metallic) && (
              <button className="btn btn-secondary" onClick={browseDest}>
                <FolderOpen size={13} /> Browse
              </button>
            )}
          </div>

          {error && (
            <div style={{ color: 'var(--mm-red)', fontSize: 'var(--mm-text-sm)', marginTop: 'var(--mm-3)', display: 'flex', alignItems: 'center', gap: 'var(--mm-2)' }}>
              <AlertTriangle size={14} />
              <strong>Error:</strong> {error}
            </div>
          )}

          {result && (
            <div className="fade-in" style={{
              marginTop: 'var(--mm-3)', padding: 'var(--mm-3)',
              border: result.files > 0 ? '1px solid var(--mm-green)' : '1px solid var(--mm-amber, #f59e0b)',
              background: result.files > 0 ? 'var(--mm-green-bg)' : 'rgba(245, 158, 11, 0.1)',
              borderRadius: 8,
              display: 'flex', gap: 'var(--mm-3)', alignItems: 'center', justifyContent: 'space-between'
            }}>
              <div style={{ display: 'flex', gap: 'var(--mm-3)', alignItems: 'center' }}>
                <CheckCircle2 size={20} color={result.files > 0 ? 'var(--mm-green)' : '#f59e0b'} style={{ flexShrink: 0 }} />
                <div>
                  <div style={{ fontWeight: 600, color: result.files > 0 ? 'var(--mm-green)' : '#f59e0b' }}>
                    {result.files > 0 ? 'Restore completed!' : 'Restore completed with warnings'}
                  </div>
                  <div style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)', marginTop: 2 }}>
                    Version {result.version} → <code>{result.destination}</code>
                    {result.files > 0 && ` · ${result.files} file${result.files > 1 ? 's' : ''} restored`}
                  </div>
                </div>
              </div>

              {(window.ferrum?.openPath || window.metallic?.openPath) && (
                <button
                  className="btn btn-secondary"
                  style={{ fontSize: 'var(--mm-text-xs)', padding: '4px 10px', display: 'flex', alignItems: 'center', gap: 4 }}
                  onClick={() => (window.ferrum || window.metallic)?.openPath?.(result.destination)}
                >
                  <FolderOpen size={13} /> Open Location
                </button>
              )}
            </div>
          )}

          <div style={{ display: 'flex', gap: 'var(--mm-2)', marginTop: 'var(--mm-4)' }}>
            <button
              className="btn btn-primary"
              onClick={startRestore}
              disabled={!canRestore || restoring}
              style={{ padding: '8px 20px' }}
            >
              {restoring ? <Loader2 size={14} className="animate-spin" /> : <Download size={14} />}
              {restoring ? 'Restoring…' : 'Restore Now'}
            </button>
            {result && (
              <button className="btn btn-secondary" onClick={() => setResult(null)}>
                <X size={13} /> Clear
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Info section */}
      <div className="card" style={{ maxWidth: 720 }}>
        <div className="card-header"><span className="card-title">How restoration works</span></div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-3)' }}>
          <div style={{ display: 'flex', gap: 'var(--mm-3)', alignItems: 'flex-start' }}>
            <Shield size={16} color="var(--mm-green)" style={{ marginTop: 2, flexShrink: 0 }} />
            <div style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)' }}>
              <strong style={{ color: 'var(--mm-text)' }}>Delete protection is active.</strong> Files deleted
              from source <em>and</em> the Trash are preserved. BLAKE3 integrity is verified for every restored chunk.
            </div>
          </div>
          <div style={{ display: 'flex', gap: 'var(--mm-3)', alignItems: 'flex-start' }}>
            <AlertTriangle size={16} color="var(--mm-yellow)" style={{ marginTop: 2, flexShrink: 0 }} />
            <div style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)' }}>
              <strong style={{ color: 'var(--mm-text)' }}>If a version shows "empty",</strong> the source
              path was missing when that snapshot ran. Pick an earlier version — your data is still there.
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
