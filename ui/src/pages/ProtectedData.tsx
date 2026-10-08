import { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import {
  Shield, FolderPlus, Folder, File, MoreVertical,
  CheckCircle2, RefreshCw, AlertCircle, Trash2, Eye, Pause, Play, Loader2,
  Upload, Zap, Lock, Archive, FolderOpen, X
} from 'lucide-react';
import { api, formatBytes, timeAgo, type ProtectedItem } from '../utils/api';

export function ProtectedData() {
  const [items, setItems] = useState<ProtectedItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [addPath, setAddPath] = useState('');
  const [adding, setAdding] = useState(false);
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  const [menuPos, setMenuPos] = useState<{ top: number; left: number } | null>(null);
  const [backingUp, setBackingUp] = useState<string | null>(null);
  const [backupDone, setBackupDone] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);
  // Protection options
  const [addCompression, setAddCompression] = useState(true);
  const [addAutoBackup, setAddAutoBackup] = useState(true);
  const [addTargetReplica, setAddTargetReplica] = useState<string>('');
  const [repos, setRepos] = useState<any[]>([]);
  const buttonRefs = useRef<Record<string, HTMLButtonElement | null>>({});

  // Fetch real data from backend
  const fetchItems = async () => {
    try {
      const [data, replicaData] = await Promise.all([
        api.getItems(),
        api.getReplicas()
      ]);
      setItems(data.items);
      setRepos(replicaData.replicas);
      setError(null);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchItems();
    const interval = setInterval(fetchItems, 10000);
    return () => clearInterval(interval);
  }, []);

  const browseFolder = async () => {
    if (window.metallic?.selectFolder) {
      const folder = await window.metallic.selectFolder();
      if (folder) setAddPath(folder);
    }
  };

  const browseFile = async () => {
    if (window.metallic?.selectFile) {
      const file = await window.metallic.selectFile();
      if (file) setAddPath(file);
    }
  };

  const addItem = async () => {
    if (!addPath.trim()) return;
    setAdding(true);
    try {
      const name = addPath.split(/[/\\]/).filter(Boolean).pop() || 'Unknown';
      const result = await api.addItem(addPath.trim(), name);
      
      if (result?.item?.id && addTargetReplica) {
        await api.setItemReplica(result.item.id, addTargetReplica);
      }
      
      setAddPath('');
      setShowAdd(false);
      setSuccessMsg(`"${name}" is now protected`);
      setTimeout(() => setSuccessMsg(null), 4000);
      await fetchItems();

      // Auto-trigger first backup if enabled
      if (addAutoBackup && result?.item?.id) {
        try {
          await api.triggerBackup(result.item.id);
          setSuccessMsg(`"${name}" protected & first backup completed!`);
          setTimeout(() => setSuccessMsg(null), 4000);
        } catch {
          // First backup failed silently — scheduler will retry
        }
      }
    } catch (e: any) {
      setError(e.message);
    } finally {
      setAdding(false);
    }
  };

  const removeItem = async (id: string) => {
    try {
      await api.removeItem(id);
      setMenuOpen(null);
      setMenuPos(null);
      await fetchItems();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const backupItem = async (id: string) => {
    setBackingUp(id);
    try {
      await api.triggerBackup(id);
      setBackupDone(id);
      setTimeout(() => setBackupDone(null), 3000);
    } catch (e: any) {
      setError(`Backup failed: ${e.message}`);
    }
    setBackingUp(null);
  };

  // Close the floating menu on outside click / Escape / scroll / resize.
  useEffect(() => {
    if (!menuOpen) return;
    const onPointerDown = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (target && target.closest('[data-row-menu]')) return;
      setMenuOpen(null);
      setMenuPos(null);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { setMenuOpen(null); setMenuPos(null); }
    };
    const onScroll = () => {
      if (menuOpen) {
        const btn = buttonRefs.current[menuOpen];
        if (btn) {
          const r = btn.getBoundingClientRect();
          setMenuPos({ top: r.bottom + 4, left: r.right - 200 });
        }
      }
    };
    document.addEventListener('mousedown', onPointerDown);
    document.addEventListener('keydown', onKey);
    window.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', onScroll);
    return () => {
      document.removeEventListener('mousedown', onPointerDown);
      document.removeEventListener('keydown', onKey);
      window.removeEventListener('scroll', onScroll, true);
      window.removeEventListener('resize', onScroll);
    };
  }, [menuOpen]);

  const openMenu = (id: string) => {
    if (menuOpen === id) {
      setMenuOpen(null);
      setMenuPos(null);
      return;
    }
    const btn = buttonRefs.current[id];
    if (btn) {
      const r = btn.getBoundingClientRect();
      setMenuPos({ top: r.bottom + 4, left: r.right - 200 });
    }
    setMenuOpen(id);
  };

  if (loading) {
    return (
      <div className="empty fade-in">
        <Loader2 size={32} className="animate-spin" style={{ color: 'var(--mm-brand)' }} />
        <p style={{ marginTop: 'var(--mm-4)' }}>Loading protected items...</p>
      </div>
    );
  }

  return (
    <div className="fade-in">
      <div className="page-header">
        <div>
          <h1 className="page-title">Protected Data</h1>
          <p className="page-subtitle">
            {items.length} {items.length === 1 ? 'item' : 'items'} under continuous protection
            {error && <span style={{ color: 'var(--mm-red)', marginLeft: 'var(--mm-2)' }}>· {error}</span>}
          </p>
        </div>
        <button className="btn btn-primary" onClick={() => setShowAdd(!showAdd)}>
          <FolderPlus size={14} /> Add Protection
        </button>
      </div>

      {/* Success notification */}
      {successMsg && (
        <div className="card fade-in" style={{
          marginBottom: 'var(--mm-4)',
          borderColor: 'var(--mm-green)',
          background: 'var(--mm-green-bg)',
          display: 'flex', alignItems: 'center', gap: 'var(--mm-3)',
        }}>
          <CheckCircle2 size={18} color="var(--mm-green)" />
          <span style={{ color: 'var(--mm-green)', fontWeight: 600, flex: 1 }}>{successMsg}</span>
          <button className="btn btn-ghost" style={{ padding: 4 }} onClick={() => setSuccessMsg(null)}>
            <X size={14} />
          </button>
        </div>
      )}

      {showAdd && (
        <div className="card fade-in" style={{ marginBottom: 'var(--mm-4)', borderColor: 'var(--mm-brand)' }}>
          <div style={{ fontSize: 'var(--mm-text-md)', fontWeight: 600, marginBottom: 'var(--mm-3)' }}>
            Protect New Item
          </div>
          <p style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)', marginBottom: 'var(--mm-3)' }}>
            Choose a folder or file to protect. Ferrum will create versioned, deduplicated backups.
          </p>

          {/* Path input */}
          <div style={{ display: 'flex', gap: 'var(--mm-2)', marginBottom: 'var(--mm-3)' }}>
            <input
              className="input mono"
              placeholder={navigator.platform.includes('Win') ? 'C:\\Users\\you\\Documents' : '/Users/you/Documents'}
              value={addPath}
              onChange={e => setAddPath(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && addItem()}
              autoFocus
              disabled={adding}
            />
            {window.metallic && (
              <div style={{ display: 'flex', gap: 'var(--mm-2)' }}>
                <button className="btn btn-secondary" onClick={browseFolder}>
                  <FolderOpen size={13} /> Folder
                </button>
                <button className="btn btn-secondary" onClick={browseFile}>
                  <File size={13} /> File
                </button>
              </div>
            )}
          </div>

          {/* Quick-pick common folders */}
          <div style={{ marginBottom: 'var(--mm-3)' }}>
            <div style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)', marginBottom: 'var(--mm-2)' }}>
              Quick pick:
            </div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 'var(--mm-2)' }}>
              {[
                { label: '📁 Documents', path: '/Users/sagarm/Documents' },
                { label: '🖥 Desktop', path: '/Users/sagarm/Desktop' },
                { label: '📥 Downloads', path: '/Users/sagarm/Downloads' },
                { label: '🎬 Movies', path: '/Users/sagarm/Movies' },
                { label: '🎵 Music', path: '/Users/sagarm/Music' },
                { label: '📷 Pictures', path: '/Users/sagarm/Pictures' },
              ].map(q => (
                <button
                  key={q.path}
                  className="btn btn-ghost"
                  style={{
                    padding: '4px 10px',
                    fontSize: 'var(--mm-text-xs)',
                    borderRadius: 'var(--mm-radius-full)',
                    border: '1px solid var(--mm-border)',
                    background: addPath === q.path ? 'var(--mm-brand-light)' : undefined,
                    borderColor: addPath === q.path ? 'var(--mm-brand)' : undefined,
                    color: addPath === q.path ? 'var(--mm-brand)' : undefined,
                  }}
                  onClick={() => setAddPath(q.path)}
                >
                  {q.label}
                </button>
              ))}
            </div>
          </div>

          {/* Advanced Settings */}
          <div style={{ marginBottom: 'var(--mm-3)', border: '1px solid var(--mm-border)', borderRadius: 'var(--mm-radius)', padding: 'var(--mm-3)' }}>
            <div style={{ fontSize: 'var(--mm-text-sm)', fontWeight: 600, marginBottom: 'var(--mm-2)' }}>Advanced Settings</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-2)' }}>
              <div>
                <label style={{ fontSize: 'var(--mm-text-xs)', color: 'var(--mm-text-muted)', display: 'block', marginBottom: 4 }}>Target Repository</label>
                <select 
                  className="input" 
                  value={addTargetReplica}
                  onChange={e => setAddTargetReplica(e.target.value)}
                  style={{ padding: '8px', cursor: 'pointer' }}
                >
                  <option value="">Local Storage (Default)</option>
                  {repos.filter(r => r.enabled).map(r => (
                    <option key={r.id} value={r.id}>{r.name} ({r.path})</option>
                  ))}
                </select>
              </div>
            </div>
          </div>

          {/* Options */}
          <div style={{ display: 'flex', gap: 'var(--mm-4)', marginBottom: 'var(--mm-3)', flexWrap: 'wrap' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-2)', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={addCompression}
                onChange={e => setAddCompression(e.target.checked)}
                style={{ accentColor: '#00B4D8' }}
              />
              <Archive size={12} color="var(--mm-text-secondary)" />
              <span style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)' }}>
                LZ4 Compression
              </span>
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-2)', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={addAutoBackup}
                onChange={e => setAddAutoBackup(e.target.checked)}
                style={{ accentColor: '#00B4D8' }}
              />
              <Zap size={12} color="var(--mm-text-secondary)" />
              <span style={{ fontSize: 'var(--mm-text-sm)', color: 'var(--mm-text-secondary)' }}>
                Backup immediately
              </span>
            </label>
          </div>

          <div style={{ display: 'flex', gap: 'var(--mm-2)' }}>
            <button className="btn btn-primary" onClick={addItem} disabled={adding || !addPath.trim()}>
              {adding ? <Loader2 size={14} className="animate-spin" /> : <Shield size={14} />}
              {adding ? 'Protecting…' : 'Protect'}
            </button>
            <button className="btn btn-secondary" onClick={() => { setShowAdd(false); setAddPath(''); }}>Cancel</button>
          </div>
        </div>
      )}

      {items.length === 0 ? (
        <div className="empty">
          <Shield size={48} />
          <h3>No protected data yet</h3>
          <p>Click "Add Protection" to start monitoring your files and folders. Ferrum will create versioned backups automatically.</p>
        </div>
      ) : (
        <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
          <div className="data-row" style={{ background: 'var(--mm-bg-elevated)', borderBottom: '1px solid var(--mm-border)' }}>
            <div style={{ width: 36 }} />
            <div className="data-info"><span style={{ fontSize: 'var(--mm-text-xs)', fontWeight: 600, color: 'var(--mm-text-muted)', textTransform: 'uppercase', letterSpacing: '0.5px' }}>Name / Path</span></div>
            <div className="data-meta"><span style={{ fontSize: 'var(--mm-text-xs)', fontWeight: 600, color: 'var(--mm-text-muted)', textTransform: 'uppercase', letterSpacing: '0.5px' }}>Type</span></div>
            <div className="data-meta" style={{ minWidth: 80 }}><span style={{ fontSize: 'var(--mm-text-xs)', fontWeight: 600, color: 'var(--mm-text-muted)', textTransform: 'uppercase', letterSpacing: '0.5px' }}>Status</span></div>
            <div className="data-meta"><span style={{ fontSize: 'var(--mm-text-xs)', fontWeight: 600, color: 'var(--mm-text-muted)', textTransform: 'uppercase', letterSpacing: '0.5px' }}>Created</span></div>
            <div style={{ width: 90 }} />
          </div>

          {items.map(item => (
            <div key={item.id} className="data-row">
              <div className={`data-icon ${item.item_type === 'folder' ? 'teal' : 'blue'}`}>
                {item.item_type === 'folder' ? <Folder size={18} /> : <File size={18} />}
              </div>
              <div className="data-info">
                <div className="data-name">{item.path.split(/[/\\]/).filter(Boolean).pop()}</div>
                <div className="data-path">{item.path}</div>
              </div>
              <div className="data-meta">
                <span style={{ textTransform: 'capitalize' }}>{item.item_type}</span>
              </div>
              <div className="data-meta" style={{ minWidth: 80 }}>
                {item.status === 'active' && <span className="badge badge-green"><CheckCircle2 size={10} /> Active</span>}
                {item.status === 'paused' && <span className="badge badge-yellow"><Pause size={10} /> Paused</span>}
                {item.status === 'unlinked' && <span className="badge badge-red"><AlertCircle size={10} /> Unlinked</span>}
              </div>
              <div className="data-meta">
                <span style={{ fontSize: 'var(--mm-text-xs)' }}>{timeAgo(item.created_at)}</span>
              </div>
              <div style={{ display: 'flex', gap: 'var(--mm-1)', justifyContent: 'flex-end', width: 90 }}>
                <button
                  className="btn btn-ghost"
                  style={{ padding: '4px 8px', fontSize: 'var(--mm-text-xs)' }}
                  onClick={() => backupItem(item.id)}
                  disabled={backingUp === item.id}
                  title="Backup now"
                >
                  {backingUp === item.id ? (
                    <Loader2 size={12} className="animate-spin" />
                  ) : backupDone === item.id ? (
                    <CheckCircle2 size={12} color="var(--mm-green)" />
                  ) : (
                    <Upload size={12} />
                  )}
                </button>
                <button
                  ref={(el) => { buttonRefs.current[item.id] = el; }}
                  data-row-menu
                  className="btn btn-ghost"
                  style={{ padding: '4px' }}
                  onClick={() => openMenu(item.id)}
                >
                  <MoreVertical size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Floating row menu */}
      {menuOpen && menuPos && createPortal(
        <div
          data-row-menu
          style={{
            position: 'fixed',
            top: menuPos.top,
            left: menuPos.left,
            zIndex: 9999,
            background: 'var(--mm-bg-elevated)',
            border: '1px solid var(--mm-border)',
            borderRadius: 'var(--mm-radius)',
            boxShadow: 'var(--mm-shadow-lg)',
            minWidth: 200,
            padding: 4,
          }}
        >
          <button
            className="btn btn-ghost"
            data-row-menu
            style={{ width: '100%', justifyContent: 'flex-start', borderRadius: 'var(--mm-radius-sm)', padding: '6px 10px' }}
            onClick={() => { backupItem(menuOpen); setMenuOpen(null); setMenuPos(null); }}
          >
            <Upload size={12} /> Backup Now
          </button>
          <button
            className="btn btn-ghost"
            data-row-menu
            style={{ width: '100%', justifyContent: 'flex-start', borderRadius: 'var(--mm-radius-sm)', padding: '6px 10px', color: 'var(--mm-red)' }}
            onClick={() => removeItem(menuOpen)}
          >
            <Trash2 size={12} /> Remove Protection
          </button>
        </div>,
        document.body
      )}
    </div>
  );
}
