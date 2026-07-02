import { useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { createProject } from '../api/client';
import {
  CloudCog,
  FileText,
  FolderGit2,
  Loader2,
  Server,
  Upload,
} from 'lucide-react';

const DEVOPS_ACCEPT =
  '.md,.markdown,.yaml,.yml,.txt,.pdf,application/pdf,text/markdown,text/plain';

const DEVOPS_PLACEHOLDER = `# DevOps 계획서 예시

## 인프라
- Docker Compose (nginx + api + redis)
- Podman 호환

## CI/CD
- GitHub Actions: lint → test → build → deploy
- PR 머지 시 자동 배포

## 모니터링
- 헬스체크 /health
- Slack 알림

## 보안
- 시크릿은 GitHub Secrets / 환경변수로 관리
`;

export function NewProjectPage() {
  const navigate = useNavigate();
  const fileRef = useRef<HTMLInputElement>(null);
  const devopsFileRef = useRef<HTMLInputElement>(null);
  const [name, setName] = useState('');
  const [repoUrl, setRepoUrl] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [devopsFile, setDevopsFile] = useState<File | null>(null);
  const [devopsText, setDevopsText] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const dropped = e.dataTransfer.files[0];
    if (dropped?.type === 'application/pdf') setFile(dropped);
    else setError('PDF 파일만 업로드할 수 있습니다.');
  };

  const hasDevops = devopsText.trim().length > 0 || devopsFile !== null;

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!file) {
      setError('PDF 계획서를 선택해주세요.');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await createProject(file, {
        name: name || undefined,
        repoUrl: repoUrl || undefined,
        devopsPlanText: devopsText || undefined,
        devopsPlanFile: devopsFile,
      });
      navigate(`/projects/${res.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : '프로젝트 생성 실패');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h2>새 프로젝트</h2>
          <p>
            PDF 외주 계획서와 함께 DevOps 계획서(CI/CD, 인프라, 배포)를
            작성하거나 업로드할 수 있습니다.
          </p>
        </div>
      </header>

      <form className="card upload-form" onSubmit={onSubmit}>
        <h3 className="section-title">
          <FileText size={18} />
          외주 계획서 (PDF) — 필수
        </h3>
        <div
          className={`dropzone ${dragOver ? 'drag-over' : ''} ${file ? 'has-file' : ''}`}
          onDragOver={(e) => {
            e.preventDefault();
            setDragOver(true);
          }}
          onDragLeave={() => setDragOver(false)}
          onDrop={onDrop}
          onClick={() => fileRef.current?.click()}
        >
          <input
            ref={fileRef}
            type="file"
            accept="application/pdf"
            hidden
            onChange={(e) => setFile(e.target.files?.[0] ?? null)}
          />
          {file ? (
            <>
              <FileText size={40} className="accent" />
              <strong>{file.name}</strong>
              <span className="muted">{(file.size / 1024).toFixed(1)} KB</span>
            </>
          ) : (
            <>
              <Upload size={40} className="muted" />
              <strong>PDF 계획서를 드래그하거나 클릭하여 선택</strong>
              <span className="muted">필수 · 최대 50MB</span>
            </>
          )}
        </div>

        <h3 className="section-title">
          <Server size={18} />
          DevOps 계획서 — 선택
        </h3>
        <p className="section-desc">
          CI/CD, Docker/K8s, 인프라, 모니터링, 배포 전략을 직접 작성하거나
          파일(.md, .yaml, .yml, .txt, .pdf)로 업로드하세요.
        </p>

        <label>
          직접 작성
          <textarea
            rows={10}
            placeholder={DEVOPS_PLACEHOLDER}
            value={devopsText}
            onChange={(e) => setDevopsText(e.target.value)}
          />
        </label>

        <div
          className={`dropzone compact ${devopsFile ? 'has-file' : ''}`}
          onClick={() => devopsFileRef.current?.click()}
        >
          <input
            ref={devopsFileRef}
            type="file"
            accept={DEVOPS_ACCEPT}
            hidden
            onChange={(e) => setDevopsFile(e.target.files?.[0] ?? null)}
          />
          {devopsFile ? (
            <>
              <CloudCog size={28} className="accent" />
              <strong>{devopsFile.name}</strong>
              <button
                type="button"
                className="btn ghost small"
                onClick={(e) => {
                  e.stopPropagation();
                  setDevopsFile(null);
                }}
              >
                제거
              </button>
            </>
          ) : (
            <>
              <CloudCog size={28} className="muted" />
              <strong>DevOps 계획서 파일 업로드 (선택)</strong>
              <span className="muted">.md · .yaml · .yml · .txt · .pdf</span>
            </>
          )}
        </div>

        {hasDevops && (
          <div className="alert info">
            DevOps 계획서가 파이프라인에 반영됩니다 — CI/CD, Dockerfile,
            compose, 인프라 설정이 자동 생성됩니다.
          </div>
        )}

        <label>
          프로젝트 이름
          <input
            type="text"
            placeholder="예: 쇼핑몰 MVP"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </label>

        <label>
          <span className="label-row">
            <FolderGit2 size={16} />
            GitHub Repo URL (선택)
          </span>
          <input
            type="url"
            placeholder="비워두면 프라이빗 레포 자동 생성"
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
          />
          <small className="hint">
            GITHUB_TOKEN 환경 변수가 설정되어 있으면 자동으로 프라이빗 레포를
            만들고, 검증 통과 후 PR을 머지합니다.
          </small>
        </label>

        {error && <div className="alert error">{error}</div>}

        <button type="submit" className="btn primary full" disabled={loading}>
          {loading ? (
            <>
              <Loader2 size={18} className="spin" />
              파이프라인 시작 중...
            </>
          ) : (
            '파이프라인 시작'
          )}
        </button>
      </form>

      <section className="card info-card">
        <h3>자동화 파이프라인</h3>
        <ol className="pipeline-list">
          <li>
            <strong>Summarize</strong> — Sonnet이 PDF + DevOps 계획서 통합 요약
          </li>
          <li>
            <strong>Architect</strong> — Fable이 아키텍처 & 인프라/CI/CD 설계
          </li>
          <li>
            <strong>Design</strong> — Stitch로 UI 디자인
          </li>
          <li>
            <strong>Implement</strong> — Codex 5.3이 코드 + DevOps 산출물 구현
          </li>
          <li>
            <strong>Verify → Debug</strong> — 품질 게이트 (최대 3회)
          </li>
          <li>
            <strong>Security → Deliver</strong> — 보안 패치 후 PR 자동 머지
          </li>
        </ol>
      </section>
    </div>
  );
}
