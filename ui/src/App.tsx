import { useState, useEffect } from 'react';
import { Sidebar } from './components/layout/Sidebar';
import { Home } from './pages/Home';
import { ProtectedData } from './pages/ProtectedData';
import { Versions } from './pages/Versions';
import { Recovery } from './pages/Recovery';
import { Storage } from './pages/Storage';
import { Security } from './pages/Security';
import { Settings } from './pages/Settings';
import { HowItWorks } from './pages/HowItWorks';
import { Assistant } from './pages/Assistant';

export type PageId = 'home' | 'protected' | 'versions' | 'recovery' | 'storage' | 'security' | 'settings' | 'flow' | 'assistant';

const PAGE_IDS: PageId[] = ['home', 'flow', 'protected', 'versions', 'recovery', 'storage', 'security', 'settings', 'assistant'];

function initialPage(): PageId {
  const h = typeof window !== 'undefined' ? window.location.hash.replace(/^#/, '') : '';
  return (PAGE_IDS.includes(h as PageId) ? (h as PageId) : 'home');
}

export default function App() {
  const [page, setPage] = useState<PageId>(initialPage);

  // Sync hash → page on hashchange (back/forward, deep links).
  useEffect(() => {
    const onHash = () => {
      const h = window.location.hash.replace(/^#/, '');
      if (PAGE_IDS.includes(h as PageId)) setPage(h as PageId);
    };
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  // Keep hash in sync with current page so reloads stay where you were.
  useEffect(() => {
    if (typeof window !== 'undefined' && window.location.hash.replace(/^#/, '') !== page) {
      window.history.replaceState(null, '', `#${page}`);
    }
  }, [page]);

  const pages: Record<PageId, JSX.Element> = {
    home: <Home />,
    flow: <HowItWorks />,
    protected: <ProtectedData />,
    versions: <Versions />,
    recovery: <Recovery />,
    storage: <Storage />,
    security: <Security />,
    settings: <Settings />,
    assistant: <Assistant />,
  };

  return (
    <div className="app-layout">
      <Sidebar currentPage={page} onNavigate={setPage} />
      <main className="main-content">
        <div className="page-content">
          {pages[page] || <Home />}
        </div>
      </main>
    </div>
  );
}
