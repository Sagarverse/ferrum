import { useState, useEffect } from 'react';
import {
  Shield, Bell, Cpu, Database, Brain, Lock, Save, RotateCcw, Check, Loader2, Sparkles
} from 'lucide-react';
import { api } from '../utils/api';
import { useBrand, BrandName } from '../context/BrandContext';

export function Settings() {
  const { brandName, setBrandName } = useBrand();
  const [settings, setSettings] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api.getSettings().then(s => { setSettings(s); setLoading(false); })
      .catch(() => {
        // Use defaults if backend unavailable
        setSettings({
          auto_start: true,
          continuous_monitoring: true,
          auto_backup: true,
          ransomware_detection: true,
          delete_protection: true,
          auto_quarantine: true,
          lockdown_password: false,
          max_cpu_percent: 30,
          max_concurrent: 4,
          pause_on_battery: true,
          compression: 'lz4',
          deduplication: true,
          retention_days: 90,
          ai_provider: 'ollama',
          ai_model: 'llama3.1:8b',
          require_confirmation: true,
          notify_backup: true,
          notify_security: true,
          notify_storage: true,
        });
        setLoading(false);
      });
  }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      await api.updateSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
    } catch {}
    setSaving(false);
  };

  const update = (key: string, value: any) => {
    setSettings((s: any) => ({ ...s, [key]: value }));
  };

  if (loading) {
    return <div className="empty fade-in"><Loader2 size={32} style={{ color: 'var(--mm-brand)', animation: 'mmPulse 1s infinite' }} /></div>;
  }

  return (
    <div className="fade-in" style={{ maxWidth: 680 }}>
      <div className="page-header">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-subtitle">Configure {brandName} behavior</p>
        </div>
        <div style={{ display: 'flex', gap: 'var(--mm-2)' }}>
          <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
            {saved ? <><Check size={13} /> Saved</> : saving ? <><Loader2 size={13} /> Saving...</> : <><Save size={13} /> Save</>}
          </button>
        </div>
      </div>

      <Group icon={Sparkles} title="Branding & Identity">
        <Select
          label="Software Name"
          desc="Choose your preferred brand title for the application"
          value={brandName}
          options={[['Ferrum', 'Ferrum'], ['Metallic Mini', 'Metallic Mini']]}
          set={v => setBrandName(v as BrandName)}
        />
      </Group>

      <Group icon={Shield} title="Protection">
        <Toggle label="Start on system boot" desc="Launch daemon automatically" on={settings.auto_start} set={v => update('auto_start', v)} />
        <Toggle label="Continuous monitoring" desc="Watch for changes in real-time" on={settings.continuous_monitoring} set={v => update('continuous_monitoring', v)} />
        <Toggle label="Automatic backups" desc="Create snapshots on detected changes" on={settings.auto_backup} set={v => update('auto_backup', v)} />
      </Group>

      <Group icon={Lock} title="Security">
        <Toggle label="Ransomware detection" desc="Monitor for suspicious encryption patterns" on={settings.ransomware_detection} set={v => update('ransomware_detection', v)} />
        <Toggle label="Delete protection" desc="Preserve versions when files are deleted" on={settings.delete_protection} set={v => update('delete_protection', v)} />
        <Toggle label="Auto-quarantine" desc="Isolate replicas during detected threats" on={settings.auto_quarantine} set={v => update('auto_quarantine', v)} />
        <Toggle label="Lockdown password" desc="Require password for destructive operations" on={settings.lockdown_password} set={v => update('lockdown_password', v)} />
      </Group>

      <Group icon={Cpu} title="Performance">
        <Slider label="Max CPU usage" desc="Limit during backup operations" value={settings.max_cpu_percent} set={v => update('max_cpu_percent', v)} unit="%" />
        <Slider label="Concurrent operations" desc="Parallel processing threads" value={settings.max_concurrent} set={v => update('max_concurrent', v)} max={16} unit="" />
        <Toggle label="Pause on battery" desc="Suspend backups when on battery power" on={settings.pause_on_battery} set={v => update('pause_on_battery', v)} />
      </Group>

      <Group icon={Database} title="Storage">
        <Select label="Compression" desc="Chunk compression algorithm" value={settings.compression}
          options={[['lz4', 'LZ4 (Fast)'], ['zstd', 'Zstd (Balanced)'], ['none', 'None']]}
          set={v => update('compression', v)} />
        <Toggle label="Deduplication" desc="Eliminate duplicate chunks" on={settings.deduplication} set={v => update('deduplication', v)} />
        <Slider label="Retention" desc="How long to keep versions" value={settings.retention_days} set={v => update('retention_days', v)} max={365} unit=" days" />
      </Group>

      <Group icon={Brain} title="AI Assistant">
        <Select label="AI Provider" desc="Model backend" value={settings.ai_provider}
          options={[['ollama', 'Ollama (Local)'], ['gemini', 'Google Gemini'], ['openai', 'OpenAI']]}
          set={v => update('ai_provider', v)} />
        <TextInput label="Model" desc="Model identifier" value={settings.ai_model} set={v => update('ai_model', v)} />
        <Toggle label="Require confirmation" desc="Confirm before destructive actions" on={settings.require_confirmation} set={v => update('require_confirmation', v)} />
      </Group>

      <Group icon={Bell} title="Notifications">
        <Toggle label="Backup completion" desc="Notify when backups finish" on={settings.notify_backup} set={v => update('notify_backup', v)} />
        <Toggle label="Security alerts" desc="Notify on suspicious activity" on={settings.notify_security} set={v => update('notify_security', v)} />
        <Toggle label="Storage warnings" desc="Alert when space is low" on={settings.notify_storage} set={v => update('notify_storage', v)} />
      </Group>
    </div>
  );
}

function Group({ icon: Icon, title, children }: { icon: any; title: string; children: React.ReactNode }) {
  return (
    <div className="setting-group">
      <div className="card">
        <div className="setting-group-title"><Icon /> {title}</div>
        {children}
      </div>
    </div>
  );
}

function Toggle({ label, desc, on, set }: { label: string; desc: string; on: boolean; set: (v: boolean) => void }) {
  return (
    <div className="setting-row">
      <div><div className="setting-label">{label}</div><div className="setting-desc">{desc}</div></div>
      <button className={`toggle ${on ? 'on' : 'off'}`} onClick={() => set(!on)}>
        <span className="toggle-knob" />
      </button>
    </div>
  );
}

function Slider({ label, desc, value, set, max = 100, unit }: {
  label: string; desc: string; value: number; set: (v: number) => void; max?: number; unit: string;
}) {
  return (
    <div className="setting-row" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 'var(--mm-1)' }}>
        <div><div className="setting-label">{label}</div><div className="setting-desc">{desc}</div></div>
        <span style={{ fontSize: 'var(--mm-text-sm)', fontWeight: 700, color: 'var(--mm-brand)', fontVariantNumeric: 'tabular-nums' }}>{value}{unit}</span>
      </div>
      <input type="range" min="1" max={max} value={value} onChange={e => set(+e.target.value)} style={{ width: '100%', accentColor: '#00B4D8', height: 4 }} />
    </div>
  );
}

function Select({ label, desc, value, options, set }: {
  label: string; desc: string; value: string; options: [string, string][]; set: (v: string) => void;
}) {
  return (
    <div className="setting-row">
      <div><div className="setting-label">{label}</div><div className="setting-desc">{desc}</div></div>
      <select className="select" value={value} onChange={e => set(e.target.value)}>
        {options.map(([val, label]) => <option key={val} value={val}>{label}</option>)}
      </select>
    </div>
  );
}

function TextInput({ label, desc, value, set }: { label: string; desc: string; value: string; set: (v: string) => void }) {
  return (
    <div className="setting-row">
      <div><div className="setting-label">{label}</div><div className="setting-desc">{desc}</div></div>
      <input className="input mono" value={value} onChange={e => set(e.target.value)} style={{ width: 180 }} />
    </div>
  );
}
