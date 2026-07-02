import { Link } from 'react-router-dom';
import { ArrowRight, FolderGit2, GitPullRequest } from 'lucide-react';
import type { Project } from '../types';

const stateLabel: Record<string, string> = {
  pending: '대기',
  running: '실행 중',
  completed: '완료',
  failed: '실패',
  cancelled: '취소됨',
};

export function ProjectCard({ project }: { project: Project }) {
  const name = project.name || `Project ${project.id.slice(0, 8)}`;

  return (
    <Link to={`/projects/${project.id}`} className="project-card">
      <div className="project-card-header">
        <div>
          <h3>{name}</h3>
          <span className={`badge state-${project.state}`}>
            {stateLabel[project.state] ?? project.state}
          </span>
        </div>
        <ArrowRight size={18} className="muted" />
      </div>

      <div className="progress-bar">
        <div
          className="progress-fill"
          style={{ width: `${project.progress_percent}%` }}
        />
      </div>
      <p className="progress-text">{project.progress_percent}% 완료</p>

      <div className="project-meta">
        {project.github_repo && (
          <span className="meta-chip">
            <FolderGit2 size={14} />
            {project.github_repo}
          </span>
        )}
        {project.pr_url && (
          <span className="meta-chip">
            <GitPullRequest size={14} />
            PR
          </span>
        )}
        {project.merge_status === 'merged' && (
          <span className="meta-chip success">Merged</span>
        )}
      </div>
    </Link>
  );
}
