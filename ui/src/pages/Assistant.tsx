import { useState, useRef, useEffect } from 'react';
import { Bot, User, Send, Shield, Loader2, Sparkles, Folder, Trash2 } from 'lucide-react';
import { api } from '../utils/api';

export function Assistant() {
  const [input, setInput] = useState('');
  const [messages, setMessages] = useState<{ role: 'user' | 'ai'; content: string; tool?: string }[]>([
    { role: 'ai', content: "Hello! I'm your Metallic Mini Assistant. I can help you protect files, list backups, or restore data. Try asking me to 'protect /path/to/folder' or 'list my files'." }
  ]);
  const [loading, setLoading] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, loading]);

  const handleSend = async () => {
    if (!input.trim()) return;
    const msg = input.trim();
    setInput('');
    setMessages(prev => [...prev, { role: 'user', content: msg }]);
    setLoading(true);

    try {
      const res = await api.chat(msg);
      
      let aiContent = "I'm sorry, I couldn't understand that.";
      let toolExecuted = '';

      try {
        const parsed = JSON.parse(res.response);
        aiContent = parsed.response || "Done.";
        
        if (parsed.action && parsed.action.type) {
          const type = parsed.action.type;
          const path = parsed.action.path;
          
          if (type === 'protect' && path) {
            await api.addItem(path, path.split(/[/\\]/).pop() || 'Item');
            toolExecuted = `Protected ${path}`;
          } else if (type === 'remove' && path) {
            const items = await api.getItems();
            const target = items.items.find((i: any) => i.path.includes(path));
            if (target) {
              await api.removeItem(target.id);
              toolExecuted = `Removed protection from ${path}`;
            } else {
              toolExecuted = `Could not find ${path} in protection list`;
            }
          } else if (type === 'list') {
            const items = await api.getItems();
            toolExecuted = `Found ${items.items.length} protected items`;
            aiContent += `\n\n${items.items.map((i: any) => `- \`${i.path}\``).join('\n')}`;
          }
        }
      } catch (parseErr) {
        // Fallback if AI didn't return perfect JSON
        aiContent = res.response;
      }

      setMessages(prev => [...prev, { role: 'ai', content: aiContent, tool: toolExecuted }]);
    } catch (e: any) {
      setMessages(prev => [...prev, { role: 'ai', content: `Error: ${e.message}` }]);
    }
    
    setLoading(false);
  };

  return (
    <div className="fade-in" style={{ height: 'calc(100vh - 120px)', display: 'flex', flexDirection: 'column' }}>
      <div className="page-header" style={{ marginBottom: 'var(--mm-4)' }}>
        <div>
          <h1 className="page-title" style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <Sparkles size={20} style={{ color: 'var(--mm-brand)' }} />
            AI Assistant
          </h1>
          <p className="page-subtitle">Control the engine with natural language.</p>
        </div>
      </div>

      <div className="card" style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: 0, overflow: 'hidden' }}>
        <div ref={scrollRef} style={{ flex: 1, overflowY: 'auto', padding: 'var(--mm-5)' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-4)' }}>
            {messages.map((m, i) => (
              <div key={i} style={{
                display: 'flex',
                gap: 'var(--mm-3)',
                flexDirection: m.role === 'user' ? 'row-reverse' : 'row'
              }}>
                <div style={{
                  width: 32, height: 32, borderRadius: 8,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  background: m.role === 'user' ? 'var(--mm-bg-hover)' : 'var(--mm-brand-light)',
                  color: m.role === 'user' ? 'var(--mm-text-secondary)' : 'var(--mm-brand)',
                  flexShrink: 0,
                }}>
                  {m.role === 'user' ? <User size={16} /> : <Bot size={16} />}
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--mm-2)', alignItems: m.role === 'user' ? 'flex-end' : 'flex-start', maxWidth: '75%' }}>
                  <div style={{
                    padding: '10px 14px',
                    borderRadius: 12,
                    background: m.role === 'user' ? 'var(--mm-brand-gradient)' : 'var(--mm-bg-elevated)',
                    color: m.role === 'user' ? '#fff' : 'var(--mm-text)',
                    fontSize: 'var(--mm-text-sm)',
                    lineHeight: 1.5,
                    border: m.role === 'user' ? 'none' : '1px solid var(--mm-border)'
                  }}>
                    {m.content}
                  </div>
                  {m.tool && (
                    <div style={{
                      display: 'flex', alignItems: 'center', gap: 6,
                      fontSize: 'var(--mm-text-xs)', color: 'var(--mm-green)',
                      background: 'var(--mm-green-bg)', padding: '4px 8px', borderRadius: 6
                    }}>
                      <Shield size={12} /> {m.tool}
                    </div>
                  )}
                </div>
              </div>
            ))}
            {loading && (
              <div style={{ display: 'flex', gap: 'var(--mm-3)' }}>
                <div style={{
                  width: 32, height: 32, borderRadius: 8,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  background: 'var(--mm-brand-light)', color: 'var(--mm-brand)'
                }}>
                  <Bot size={16} />
                </div>
                <div style={{ padding: '12px', background: 'var(--mm-bg-elevated)', borderRadius: 12, border: '1px solid var(--mm-border)' }}>
                  <Loader2 size={16} className="animate-spin" style={{ color: 'var(--mm-brand)' }} />
                </div>
              </div>
            )}
          </div>
        </div>

        <div style={{ padding: 'var(--mm-4)', borderTop: '1px solid var(--mm-border)', background: 'var(--mm-bg-surface)' }}>
          <div style={{ display: 'flex', gap: 'var(--mm-2)' }}>
            <input
              className="input"
              style={{ flex: 1 }}
              placeholder="Ask the engine to protect a folder, list your files, etc..."
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleSend()}
            />
            <button className="btn btn-primary" onClick={handleSend} disabled={loading || !input.trim()}>
              <Send size={14} /> Send
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
