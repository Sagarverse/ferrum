/** API service — talks to the real Ferrum Rust backend. */

const USE_IPC = typeof window !== 'undefined'
  && (window as any).metallic
  && typeof (window as any).metallic.api === 'function';

async function request<T>(method: string, path: string, body?: any): Promise<T> {
  if (USE_IPC) {
    return (window as any).metallic.api(method, path, body);
  }
  const opts: RequestInit = {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  };
  const res = await fetch(`/api${path}`, opts);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || `HTTP ${res.status}`);
  }
  return res.json();
}

// ─── Types ──────────────────────────────────────────────────

export interface ProtectedItem {
  id: string;
  path: string;
  item_type: string;
  policy_id: string;
  target_replica_id: string | null;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface VersionRecord {
  id: string;
  protected_item_id: string;
  version_number: number;
  manifest_hash: string;
  total_size: number;
  total_chunks: number;
  status: string;
  created_at: string;
  metadata?: string;
}

export interface ReplicaRecord {
  id: string;
  name: string;
  path: string;
  replica_type: string;
  encrypted: boolean;
  enabled: boolean;
  status: string;
  last_sync_at?: string;
  created_at: string;
}

export interface IncidentRecord {
  id: string;
  severity: string;
  threat_score: number;
  process_name?: string;
  process_path?: string;
  affected_files: number;
  description: string;
  status: string;
  created_at: string;
  resolved_at?: string;
}

export interface SystemStatus {
  protection_score: number;
  protected_items: number;
  replicas: { total: number; healthy: number };
  storage: { total_size: number; dedup_size: number; saved: number };
  security: { threat_level: string; active_incidents: number };
  recovery: { status: string };
}

export interface JobRecord {
  id: string;
  job_type: string;
  status: string;
  started_at: string;
  completed_at?: string;
  files_processed: number;
  bytes_processed: number;
  error_message?: string;
}

// ─── API Functions ──────────────────────────────────────────

export const api = {
  // Health
  health: () => request<{ status: string; version: string }>('GET', '/health'),

  // System status
  status: () => request<SystemStatus>('GET', '/status'),
  metrics: () => request<any>('GET', '/metrics'),

  // Protected items
  getItems: () => request<{ items: ProtectedItem[] }>('GET', '/protected'),
  addItem: (path: string, name?: string) =>
    request<{ item: ProtectedItem }>('POST', '/protected', { path, name }),
  removeItem: (id: string) => request<any>('DELETE', `/protected/${id}`),
  setItemReplica: (id: string, replica_id: string | null) =>
    request<any>('POST', `/protected/${id}/replica`, { replica_id }),

  // Versions
  getVersions: (itemId: string) =>
    request<{ versions: VersionRecord[] }>('GET', `/versions/${itemId}`),
  deleteVersion: (versionId: string) =>
    request<any>('DELETE', `/versions/delete/${versionId}`),

  // Replicas
  getReplicas: () => request<{ replicas: ReplicaRecord[] }>('GET', '/replicas'),
  addReplica: (name: string, path: string, encrypted: boolean) =>
    request<{ replica: ReplicaRecord }>('POST', '/replicas', { name, path, encrypted }),
  toggleReplica: (id: string, enabled: boolean) =>
    request<any>('POST', `/replicas/${id}/toggle`, { enabled }),

  // Security
  getSecurityStatus: () => request<any>('GET', '/security/status'),
  getIncidents: () => request<{ incidents: IncidentRecord[] }>('GET', '/security/incidents'),

  // Jobs
  getJobs: () => request<{ jobs: JobRecord[] }>('GET', '/jobs'),
  triggerBackup: (itemId: string) =>
    request<any>('POST', '/backup/trigger', { item_id: itemId }),

  // Recovery
  restore: (itemId: string, version: string, destination: string) =>
    request<any>('POST', '/restore', { item_id: itemId, version, destination }),

  // Settings
  getSettings: () => request<any>('GET', '/settings'),
  updateSettings: (settings: any) =>
    request<any>('PUT', '/settings', settings),

  // AI Chat
  chat: (message: string) =>
    request<{ response: string; tool_calls?: any[]; ai_available?: boolean }>('POST', '/ai/chat', { message }),

  // System
  resetAll: () => request<any>('POST', '/reset'),
};

// ─── Helpers ────────────────────────────────────────────────

export function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function timeAgo(dateStr: string): string {
  if (!dateStr) return 'never';
  const sec = Math.floor((Date.now() - new Date(dateStr).getTime()) / 1000);
  if (sec < 0) return 'just now';
  if (sec < 60) return 'just now';
  if (sec < 3600) return `${Math.floor(sec / 60)}m ago`;
  if (sec < 86400) return `${Math.floor(sec / 3600)}h ago`;
  return `${Math.floor(sec / 86400)}d ago`;
}

export function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit',
  });
}

// ─── Live event stream ──────────────────────────────────────
//
// subscribeEvents(cb) returns an unsubscribe function.
// The handler is called for every engine event (file_processed, backup_started,
// backup_completed, restore_*, heartbeat, …). `payload.event` is the SSE event
// name (e.g. "file_processed"); `payload.data` is the parsed JSON body.

export type EngineEventHandler = (payload: { event: string; data: any }) => void;

export async function subscribeEvents(handler: EngineEventHandler): Promise<() => void> {
  // Electron / packaged: use the IPC bridge so we go through preload.
  if (typeof window !== 'undefined' && (window as any).metallic?.subscribeEvents) {
    return (window as any).metallic.subscribeEvents({ onEvent: handler });
  }
  // Plain browser dev: fall back to EventSource pointing at the daemon.
  const url = 'http://127.0.0.1:19270/api/events/stream';
  const es = new EventSource(url);
  const dispatch = (name: string) => (e: MessageEvent) => {
    try {
      handler({ event: name, data: JSON.parse(e.data) });
    } catch {
      handler({ event: name, data: e.data });
    }
  };
  const names = [
    'backup_started', 'file_processed', 'backup_completed',
    'restore_started', 'restore_file_written', 'restore_completed',
    'heartbeat', 'message',
  ];
  const listeners = names.map((n) => {
    const fn = dispatch(n);
    es.addEventListener(n, fn);
    return [n, fn] as const;
  });
  return () => {
    for (const [n, fn] of listeners) es.removeEventListener(n, fn);
    es.close();
  };
}
