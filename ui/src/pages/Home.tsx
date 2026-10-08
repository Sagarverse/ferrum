import { useState, useRef, useEffect } from 'react';
import { Logo } from '../components/Logo';
import {
  Shield, HardDrive, Clock, Database, Send, FolderPlus,
  HeartPulse, RotateCcw, FileSearch, ShieldCheck, Zap, Activity,
  AlertTriangle, Wifi, WifiOff, Info, ArrowRight
} from 'lucide-react';
import { api, formatBytes, type SystemStatus } from '../utils/api';

interface ChatMessage {
  id: string;
  role: 'user' | 'ai';
  content: string;
}

const CHIPS = [
  { icon: FolderPlus, text: 'Protect my Documents folder' },
  { icon: HeartPulse, text: 'Show system health' },
  { icon: Clock, text: 'List recent backups' },
  { icon: RotateCcw, text: 'Recover deleted files' },
  { icon: FileSearch, text: 'Create backup policy' },
];

export function Home() {
  const [msgs, setMsgs] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [backendOnline, setBackendOnline] = useState(false);
  const endRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Fetch real status from backend
  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const s = await api.status();
        setStatus(s);
        setBackendOnline(true);
      } catch {
        setBackendOnline(false);
      }
    };
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000); // refresh every 5s
    return () => clearInterval(interval);
  }, []);

  useEffect(() => { endRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [msgs]);

  const send = async (text?: string) => {
    const msg = (text || input).trim();
    if (!msg) return;
    setMsgs(p => [...p, { id: `u${Date.now()}`, role: 'user', content: msg }]);
    setInput('');
    setLoading(true);

    let response: string = '';
    try {
      const lower = msg.toLowerCase();
      
      // Free "AI" command parsing
      if (lower.includes('protect') && (lower.includes('/') || lower.includes('\\'))) {
        const path = msg.match(/(?:\/|[a-zA-Z]:\\)[^\s"']+/)?.[0];
        if (path) {
          await api.addItem(path, path.split(/[/\\]/).pop() || 'Item');
          response = `**Done!** I have added \`${path}\` to your protected items. I will monitor it for changes.`;
        } else {
          response = `I see you want to protect a file. Please provide the exact path (e.g., "protect /Users/you/folder").`;
        }
      } else if (lower.includes('list')) {
        const items = await api.getItems();
        if (items.items.length === 0) {
          response = `You don't have any protected items yet.`;
        } else {
          response = `Here are your protected items:\n\n${items.items.map((i: any) => `- \`${i.path}\``).join('\n')}`;
        }
      } else if (lower.includes('remove') || lower.includes('delete') || lower.includes('stop')) {
        const path = msg.match(/(?:\/|[a-zA-Z]:\\)[^\s"']+/)?.[0];
        if (path) {
          const items = await api.getItems();
          const target = items.items.find((i: any) => i.path === path || i.path.endsWith(path));
          if (target) {
            await api.removeItem(target.id);
            response = `**Done!** I have removed \`${target.path}\` from protection.`;
          } else {
            response = `I couldn't find an item matching \`${path}\` in your protected list.`;
          }
        } else if (lower.includes('all')) {
            const items = await api.getItems();
            for (const item of items.items) {
                await api.removeItem(item.id);
            }
            response = `**Done!** I have removed all items from protection.`;
        } else {
          response = `Please specify the path of the item you want to remove (e.g., "remove /Users/you/folder").`;
        }
      } else {
        // Fallback to backend API
        const data = await api.chat(msg);
        response = data.response;
        
        // Add a helpful nudge if they just typed gibberish
        if (lower.length < 5 && !lower.includes('hi')) {
            response = "I am ready! Try saying **protect /Users/sagarm/Space** or **list my items** or **delete /Users/sagarm/Space**.";
        }
      }
    } catch {
      response = 'Backend is starting up. Please try again in a moment.';
    }

    setMsgs(p => [...p, { id: `a${Date.now()}`, role: 'ai', content: response }]);
    setLoading(false);
    inputRef.current?.focus();
  };

  const score = status?.protection_score ?? 0;
  const scoreColor = score >= 80 ? 'var(--mm-green)' : score >= 50 ? 'var(--mm-yellow)' : 'var(--mm-red)';
  const circ = 2 * Math.PI * 50;
  const offset = circ - (score / 100) * circ;
  const totalSize = status?.storage?.total_size ?? 0;
  const dedupSize = status?.storage?.dedup_size ?? 0;
  const savedSize = status?.storage?.saved ?? 0;
  const dedupRatio = dedupSize > 0 ? (totalSize / dedupSize).toFixed(1) : '—';

  return (
    <div className="chat-layout">
      <div className="chat-main">
        <div className="chat-messages">
          {msgs.length === 0 && (
            <div className="chat-welcome fade-in">
              <div className="chat-welcome-logo"><Logo size={48} /></div>
              <h2>Metallic Mini Assistant</h2>
              <p>Ask me to protect data, check backups, or configure policies.</p>
              <div className="chat-chips">
                {CHIPS.map(c => (
                  <button key={c.text} className="chat-chip" onClick={() => send(c.text)}>
                    <c.icon /> {c.text}
                  </button>
                ))}
              </div>
            </div>
          )}
          {msgs.map(m => (
            <div key={m.id} className={`chat-msg ${m.role === 'user' ? 'user' : 'ai'}`}>
              <div className={`chat-avatar ${m.role === 'ai' ? 'ai' : 'human'}`}>
                {m.role === 'ai' ? 'M' : 'U'}
              </div>
              <div className="chat-bubble">
                {m.content.split('\n').map((line, i) => {
                  if (!line.trim()) return <br key={i} />;
                  const parts = line.split(/(\*\*[^*]+\*\*)/g);
                  return (
                    <p key={i} style={{ margin: '2px 0' }}>
                      {parts.map((p, j) =>
                        p.startsWith('**') && p.endsWith('**')
                          ? <strong key={j}>{p.slice(2, -2)}</strong>
                          : p
                      )}
                    </p>
                  );
                })}
              </div>
            </div>
          ))}
          {loading && (
            <div className="chat-msg ai">
              <div className="chat-avatar ai">M</div>
              <div className="chat-bubble"><div className="loading-dots"><span /><span /><span /></div></div>
            </div>
          )}
          <div ref={endRef} />
        </div>

        <div className="chat-input-bar">
          <div className="chat-input-wrap">
            <input
              ref={inputRef}
              className="chat-input"
              placeholder="Ask Metallic Mini..."
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); } }}
              disabled={loading}
            />
            <button className="chat-send" onClick={() => send()} disabled={!input.trim() || loading}>
              <Send size={14} />
            </button>
          </div>
        </div>
      </div>

      {/* Right — Real Status Panel */}
      <div className="status-panel fade-in">
        {/* Connection Status */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--mm-2)', marginBottom: 'var(--mm-4)', fontSize: 'var(--mm-text-xs)' }}>
          {backendOnline
            ? <><Wifi size={12} color="var(--mm-green)" /><span style={{ color: 'var(--mm-green)' }}>Engine connected</span></>
            : <><WifiOff size={12} color="var(--mm-red)" /><span style={{ color: 'var(--mm-red)' }}>Engine offline</span></>
          }
        </div>

        {/* Protection Score */}
        <div className="status-section" style={{ display: 'flex', justifyContent: 'center' }}>
          <div className="score-ring">
            <svg width="120" height="120" viewBox="0 0 120 120">
              <circle className="track" cx="60" cy="60" r="50" />
              <circle className="value" cx="60" cy="60" r="50"
                stroke={scoreColor}
                strokeDasharray={circ}
                strokeDashoffset={offset} />
            </svg>
            <div className="score-center">
              <span className="score-num" style={{ color: scoreColor }}>{score}</span>
              <span className="score-lbl">Protection</span>
            </div>
          </div>
        </div>

        <div style={{ textAlign: 'center', marginBottom: 'var(--mm-4)' }}>
          <div style={{ 
            fontSize: '11px', 
            color: 'var(--mm-text-muted)', 
            marginBottom: 12,
            background: 'rgba(255,255,255,0.03)',
            padding: '8px',
            borderRadius: '6px',
            display: 'inline-block'
          }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 4, marginBottom: 4 }}>
              <Info size={12} /> <span>Score Breakdown:</span>
            </div>
            <div>Base(50) + Protection(10) + Replica(15) + Health(15) = 90</div>
          </div>
          {score < 100 && (
            <button 
              className="btn btn-secondary" 
              style={{ fontSize: 'var(--mm-text-xs)', padding: '4px 12px', margin: '0 auto' }}
              onClick={() => window.location.hash = '#storage'}
            >
              Add 2nd Replica to reach 100 <ArrowRight size={12} />
            </button>
          )}
        </div>

        <div className="status-section">
          <div className="status-section-title">Live Status</div>
          <div className="status-row">
            <span className="status-row-label"><Shield size={14} /> Protected</span>
            <span className="status-row-value">{status?.protected_items ?? 0} items</span>
          </div>
          <div className="status-row">
            <span className="status-row-label"><HardDrive size={14} /> Replicas</span>
            <span className="status-row-value">
              {status?.replicas ? `${status.replicas.healthy} / ${status.replicas.total}` : '0 / 0'}
            </span>
          </div>
          <div className="status-row">
            <span className="status-row-label"><ShieldCheck size={14} /> Threats</span>
            {status?.security?.active_incidents === 0
              ? <span className="badge badge-green">None</span>
              : <span className="badge badge-red">{status?.security?.active_incidents}</span>
            }
          </div>
          <div className="status-row">
            <span className="status-row-label"><Activity size={14} /> Threat Level</span>
            <span className="status-row-value" style={{ textTransform: 'capitalize' }}>
              {status?.security?.threat_level ?? 'unknown'}
            </span>
          </div>
        </div>

        <div className="status-section">
          <div className="status-section-title">Storage</div>
          <div className="status-row">
            <span className="status-row-label"><Database size={14} /> Original</span>
            <span className="status-row-value">{formatBytes(totalSize)}</span>
          </div>
          <div className="status-row">
            <span className="status-row-label"><Zap size={14} /> Stored</span>
            <span className="status-row-value">{formatBytes(dedupSize)}</span>
          </div>
          <div className="status-row">
            <span className="status-row-label" style={{ color: 'var(--mm-green)' }}>Saved</span>
            <span className="status-row-value" style={{ color: 'var(--mm-green)' }}>{formatBytes(savedSize)}</span>
          </div>
          <div className="status-row">
            <span className="status-row-label">Dedup Ratio</span>
            <span className="status-row-value">{dedupRatio}x</span>
          </div>
        </div>
      </div>
    </div>
  );
}
