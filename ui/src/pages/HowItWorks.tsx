// How It Works — simplified, clean pipeline visualization.
// Shows the journey of a file through the protection pipeline with
// clear stages and live event streaming.

import { useEffect, useMemo, useState } from 'react';
import {
  Eye, Scissors, Hash, Layers, Lock, Save, Copy, GitCommit,
  Play, RotateCcw, CheckCircle2, Loader2, Zap, Activity,
  ChevronRight, Sparkles, Archive
} from 'lucide-react';
import { api, formatBytes, subscribeEvents } from '../utils/api';

interface Stage {
  id: string;
  label: string;
  hint: string;
  detail: string;
  icon: any;
  color: string;
}

const STAGES: Stage[] = [
  { id: 'watch', label: 'Watch', hint: 'Detect changes',
    detail: 'File system watcher detects new and changed files in your protected folders.',
    icon: Eye, color: 'var(--mm-brand)' },
  { id: 'chunk', label: 'Chunk', hint: 'FastCDC split',
    detail: 'Content-defined chunking splits files into ~256KB blocks. Only changed regions are re-processed.',
    icon: Scissors, color: '#A855F7' },
  { id: 'hash', label: 'Hash', hint: 'BLAKE3',
    detail: 'Each chunk gets a BLAKE3 fingerprint — used as its address in the store.',
    icon: Hash, color: 'var(--mm-blue)' },
  { id: 'dedup', label: 'Dedup', hint: 'Skip duplicates',
    detail: 'If a chunk with this hash already exists, it\'s referenced — not stored again.',
    icon: Layers, color: 'var(--mm-yellow)' },
  { id: 'compress', label: 'Compress', hint: 'LZ4/Zstd',
    detail: 'Unique chunks are compressed before encryption — smaller payloads, less disk usage.',
    icon: Archive, color: '#A855F7' },
  { id: 'encrypt', label: 'Encrypt', hint: 'AES-256-GCM',
    detail: 'Compressed chunks are sealed with AES-256-GCM. Keys derived via Argon2id.',
    icon: Lock, color: 'var(--mm-red)' },
  { id: 'store', label: 'Store', hint: 'Content-addressed',
    detail: 'Chunks land in a sharded directory: chunks/<hash prefix>/<hash>.',
    icon: Save, color: 'var(--mm-blue)' },
  { id: 'replicate', label: 'Replicate', hint: 'N copies',
    detail: 'Chunks are mirrored to configured replicas — local drives, external SSDs, or cloud.',
    icon: Copy, color: 'var(--mm-green)' },
  { id: 'version', label: 'Version', hint: 'Immutable snapshot',
    detail: 'A new version record is created. Past versions stay forever for point-in-time recovery.',
    icon: GitCommit, color: 'var(--mm-green)' },
];

type StageState = 'idle' | 'active' | 'done';

export function HowItWorks() {
  const [stageStates, setStageStates] = useState<Record<string, StageState>>(
    () => Object.fromEntries(STAGES.map(s => [s.id, 'idle']))
  );
  const [isBackingUp, setIsBackingUp] = useState(false);
  const [lastBackup, setLastBackup] = useState<{ version: number; files: number; bytes: number; durationMs: number } | null>(null);
  const [totals, setTotals] = useState({ files: 0, bytes: 0, versions: 0, replicas: 0 });
  const [eventLog, setEventLog] = useState<Array<{ ts: number; event: string; text: string }>>([]);

  const pushStage = (id: string, state: StageState) =>
    setStageStates(prev => ({ ...prev, [id]: state }));

  const addLog = (event: string, text: string) =>
    setEventLog(prev => [{ ts: Date.now(), event, text }, ...prev].slice(0, 20));

  // SSE subscription
  useEffect(() => {
    let unsub: (() => void) | undefined;
    let cancelled = false;

    subscribeEvents(({ event, data }) => {
      if (cancelled) return;

      if (event === 'backup_started') {
        setIsBackingUp(true);
        setLastBackup(null);
        setStageStates(Object.fromEntries(STAGES.map(s => [s.id, 'idle'])));
        pushStage('watch', 'active');
        addLog(event, `Backup started: ${data.source_path}`);
      } else if (event === 'file_processed') {
        // Cascade through stages with timing
        const order = ['watch','chunk','hash','dedup','compress','encrypt','store','replicate','version'];
        order.forEach((s, i) => {
          setTimeout(() => {
            if (i > 0) pushStage(order[i - 1], 'done');
            pushStage(s, 'active');
          }, i * 60);
        });
        setTimeout(() => pushStage('version', 'done'), order.length * 60);
        addLog(event, `${data.relative_path} · ${formatBytes(data.size)} · ${data.chunks} chunks`);
      } else if (event === 'backup_completed') {
        setStageStates(Object.fromEntries(STAGES.map(s => [s.id, 'done'])));
        setIsBackingUp(false);
        setLastBackup({
          version: data.version,
          files: data.files,
          bytes: data.bytes,
          durationMs: data.duration_ms,
        });
        addLog(event, `v${data.version} · ${data.files} files · ${formatBytes(data.bytes)} · ${(data.duration_ms / 1000).toFixed(2)}s`);
      } else if (event === 'restore_completed') {
        addLog(event, `Restored v${data.version} → ${data.destination}`);
      }
    }).then(u => { if (!cancelled) unsub = u; });

    return () => { cancelled = true; unsub?.(); };
  }, []);

  // Poll totals
  useEffect(() => {
    let cancelled = false;
    const tick = async () => {
      try {
        const [status, replicas] = await Promise.all([api.status(), api.getReplicas()]);
        if (cancelled) return;
        setTotals({
          files: status.protected_items ?? 0,
          bytes: status.storage?.total_size ?? 0,
          versions: 0,
          replicas: replicas.replicas?.length ?? 0,
        });
      } catch {}
    };
    tick();
    const id = setInterval(tick, 8000);
    return () => { cancelled = true; clearInterval(id); };
  }, []);

  const triggerBackup = async () => {
    try {
      const items = await api.getItems();
      if (!items.items?.length) {
        alert('No protected items. Add a folder first in "Protected Data".');
        return;
      }
      await api.triggerBackup(items.items[0].id);
    } catch (e: any) {
      alert('Backup failed: ' + (e.message || 'unknown'));
    }
  };

  const overallState = useMemo<StageState>(() => {
    const states = Object.values(stageStates);
    if (states.every(s => s === 'done')) return 'done';
    if (states.some(s => s === 'active')) return 'active';
    return 'idle';
  }, [stageStates]);

  return (
    <div className="fade-in" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-4)' }}>
      {/* Header */}
      <div className="page-header">
        <div>
          <h1 className="page-title" style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <Activity size={20} style={{ color: 'var(--mm-brand)' }} />
            How It Works
          </h1>
          <p className="page-subtitle">
            Watch the real-time pipeline as Ferrum protects your data.
          </p>
        </div>
        <button className="btn btn-primary" onClick={triggerBackup} disabled={isBackingUp}>
          {isBackingUp ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />}
          {isBackingUp ? 'Running…' : 'Back up now'}
        </button>
      </div>

      {/* Status banner */}
      <div className="card" style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-3)' }}>
        <Sparkles size={16} style={{ color: 'var(--mm-brand)', flexShrink: 0 }} />
        <div style={{ flex: 1, fontSize: 'var(--mm-text-sm)' }}>
          {isBackingUp ? (
            <><strong style={{ color: 'var(--mm-brand)' }}>Backup in progress…</strong> Watch the pipeline below</>
          ) : lastBackup ? (
            <>
              <strong style={{ color: 'var(--mm-green)' }}>Last backup complete</strong>
              <span style={{ color: 'var(--mm-text-muted)', marginLeft: 8 }}>
                v{lastBackup.version} · {lastBackup.files} files · {formatBytes(lastBackup.bytes)} · {(lastBackup.durationMs / 1000).toFixed(2)}s
              </span>
            </>
          ) : (
            <><strong>Engine idle</strong><span style={{ color: 'var(--mm-text-muted)', marginLeft: 8 }}>Click "Back up now" to see the pipeline in action</span></>
          )}
        </div>
        <span className="badge" style={{
          background: overallState === 'active' ? 'var(--mm-brand-light)' : overallState === 'done' ? 'var(--mm-green-bg)' : 'var(--mm-bg-hover)',
          color: overallState === 'active' ? 'var(--mm-brand)' : overallState === 'done' ? 'var(--mm-green)' : 'var(--mm-text-muted)',
          border: `1px solid ${overallState === 'active' ? 'var(--mm-brand)' : overallState === 'done' ? 'var(--mm-green)' : 'var(--mm-border)'}`,
        }}>
          {overallState === 'active' ? <><Loader2 size={10} className="animate-spin" /> streaming</> :
           overallState === 'done' ? <><CheckCircle2 size={10} /> done</> :
           <><Zap size={10} /> idle</>}
        </span>
      </div>

      {/* Pipeline — clean horizontal flow */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">Data Pipeline</div>
        </div>
        <div className="pipeline-flow">
          {STAGES.map((stage, i) => {
            const state = stageStates[stage.id];
            const Icon = stage.icon;
            return (
              <div key={stage.id} className="pipeline-flow-item">
                <div
                  className={`pipeline-node ${state}`}
                  style={{
                    '--node-color': stage.color,
                    borderColor: state === 'active' ? stage.color : state === 'done' ? 'var(--mm-green)' : 'var(--mm-border)',
                  } as React.CSSProperties}
                  title={stage.detail}
                >
                  <div className="pipeline-node-icon">
                    {state === 'active' ? <Loader2 size={16} className="animate-spin" /> :
                     state === 'done' ? <CheckCircle2 size={16} /> :
                     <Icon size={16} />}
                  </div>
                  <div className="pipeline-node-label">{stage.label}</div>
                  <div className="pipeline-node-hint">{stage.hint}</div>
                </div>
                {i < STAGES.length - 1 && (
                  <div className={`pipeline-flow-arrow ${state === 'done' ? 'done' : ''}`}>
                    <ChevronRight size={14} />
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Bottom: explanations + event log */}
      <div className="grid" style={{ gridTemplateColumns: '1fr 1fr' }}>
        {/* Stage explanations */}
        <div className="card">
          <div className="card-header">
            <div className="card-title">Stage Details</div>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-1)' }}>
            {STAGES.map(s => {
              const Icon = s.icon;
              return (
                <div key={s.id} className="defense-item" style={{ padding: 'var(--mm-2) var(--mm-3)' }}>
                  <div className="defense-icon" style={{ background: `${s.color}15`, color: s.color, width: 26, height: 26 }}>
                    <Icon size={13} />
                  </div>
                  <div className="defense-info">
                    <div className="defense-name">{s.label} <span style={{ color: 'var(--mm-text-muted)', fontWeight: 400 }}>— {s.hint}</span></div>
                    <div className="defense-desc">{s.detail}</div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Live event log */}
        <div className="card">
          <div className="card-header">
            <div className="card-title">Live Engine Feed</div>
            <span className="badge badge-blue">{eventLog.length}</span>
          </div>
          <div style={{ maxHeight: 400, overflowY: 'auto', fontFamily: 'var(--mm-font-mono)', fontSize: 'var(--mm-text-xs)' }}>
            {eventLog.length === 0 ? (
              <div style={{ color: 'var(--mm-text-muted)', padding: 'var(--mm-4)', textAlign: 'center' }}>
                No events yet. Trigger a backup to see real-time data flow.
              </div>
            ) : eventLog.map((e, i) => (
              <div key={i} style={{
                borderLeft: `2px solid ${
                  e.event === 'backup_started' ? 'var(--mm-brand)' :
                  e.event === 'backup_completed' ? 'var(--mm-green)' :
                  e.event === 'file_processed' ? '#A855F7' : 'var(--mm-text-muted)'
                }`,
                padding: '5px 8px',
                marginBottom: 3,
                background: 'var(--mm-bg-app)',
                borderRadius: '0 4px 4px 0',
              }}>
                <div style={{ fontWeight: 600, color: 'var(--mm-text-secondary)' }}>
                  {e.event}
                  <span style={{ color: 'var(--mm-text-muted)', fontWeight: 400, marginLeft: 8 }}>
                    {new Date(e.ts).toLocaleTimeString('en-US', { hour12: false })}
                  </span>
                </div>
                <div style={{ color: 'var(--mm-text-muted)', marginTop: 2, wordBreak: 'break-all' }}>
                  {e.text}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
