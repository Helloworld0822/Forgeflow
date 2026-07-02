import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { listProjects } from '../api/client';
import { ProjectCard } from '../components/ProjectCard';
import type { Project } from '../types';
import { Plus, RefreshCw } from 'lucide-react';

export function DashboardPage() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      const data = await listProjects();
      setProjects(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : '목록 로드 실패');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    const timer = setInterval(load, 5000);
    return () => clearInterval(timer);
  }, []);

  const running = projects.filter((p) => p.state === 'running').length;
  const completed = projects.filter((p) => p.state === 'completed').length;

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h2>대시보드</h2>
          <p>PDF 계획서 업로드부터 GitHub PR 머지까지 자동화된 파이프라인</p>
        </div>
        <div className="header-actions">
          <button className="btn ghost" onClick={load} disabled={loading}>
            <RefreshCw size={16} className={loading ? 'spin' : ''} />
            새로고침
          </button>
          <Link to="/new" className="btn primary">
            <Plus size={16} />
            새 프로젝트
          </Link>
        </div>
      </header>

      <section className="stats-row">
        <div className="stat-card">
          <span className="stat-label">전체 프로젝트</span>
          <strong>{projects.length}</strong>
        </div>
        <div className="stat-card">
          <span className="stat-label">실행 중</span>
          <strong className="accent">{running}</strong>
        </div>
        <div className="stat-card">
          <span className="stat-label">완료</span>
          <strong className="success">{completed}</strong>
        </div>
      </section>

      {error && <div className="alert error">{error}</div>}

      <section className="card">
        <h3>프로젝트 목록</h3>
        {loading && projects.length === 0 ? (
          <p className="muted">로딩 중...</p>
        ) : projects.length === 0 ? (
          <div className="empty-state">
            <p>아직 프로젝트가 없습니다.</p>
            <Link to="/new" className="btn primary">
              첫 프로젝트 시작하기
            </Link>
          </div>
        ) : (
          <div className="project-grid">
            {projects.map((p) => (
              <ProjectCard key={p.id} project={p} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
