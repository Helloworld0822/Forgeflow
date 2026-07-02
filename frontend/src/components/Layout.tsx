import {
  Activity,
  Box,
  FolderGit2,
  GitPullRequest,
  Layers,
  Sparkles,
  Upload,
  Zap,
} from 'lucide-react';
import { Link, Outlet, useLocation } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { getHealth } from '../api/client';
import type { HealthResponse } from '../types';

export function Layout() {
  const location = useLocation();
  const [health, setHealth] = useState<HealthResponse | null>(null);

  useEffect(() => {
    getHealth().then(setHealth).catch(() => setHealth(null));
  }, []);

  const nav = [
    { to: '/', label: '대시보드', icon: Layers },
    { to: '/new', label: '새 프로젝트', icon: Upload },
  ];

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-icon">
            <Sparkles size={20} />
          </div>
          <div>
            <h1>AutoForge</h1>
            <p>AI 외주 자동화</p>
          </div>
        </div>

        <nav className="nav">
          {nav.map(({ to, label, icon: Icon }) => (
            <Link
              key={to}
              to={to}
              className={location.pathname === to ? 'nav-link active' : 'nav-link'}
            >
              <Icon size={18} />
              {label}
            </Link>
          ))}
        </nav>

        <div className="sidebar-footer">
          <div className="status-grid">
            <StatusPill
              icon={<Activity size={14} />}
              label="API"
              ok={health?.status === 'ok'}
            />
            <StatusPill
              icon={<FolderGit2 size={14} />}
              label="GitHub"
              ok={health?.github ?? false}
            />
            <StatusPill
              icon={<GitPullRequest size={14} />}
              label="Auto Merge"
              ok={health?.github_auto_merge ?? false}
            />
            <StatusPill
              icon={<Zap size={14} />}
              label="MQ"
              ok={health?.message_queue ?? false}
            />
          </div>
        </div>
      </aside>

      <main className="main">
        <Outlet />
      </main>
    </div>
  );
}

function StatusPill({
  icon,
  label,
  ok,
}: {
  icon: React.ReactNode;
  label: string;
  ok: boolean;
}) {
  return (
    <div className={`status-pill ${ok ? 'ok' : 'off'}`}>
      {icon}
      <span>{label}</span>
      <Box size={8} className="dot" />
    </div>
  );
}
