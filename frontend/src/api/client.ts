import type {
  CreateProjectResponse,
  HealthResponse,
  Project,
  ProjectDetail,
} from '../types';

const API_BASE = '';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, init);
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export async function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>('/health');
}

export async function listProjects(): Promise<Project[]> {
  return request<Project[]>('/v1/projects');
}

export async function getProject(id: string): Promise<ProjectDetail> {
  return request<ProjectDetail>(`/v1/projects/${id}`);
}

export async function createProject(
  file: File,
  name?: string,
  repoUrl?: string,
): Promise<CreateProjectResponse> {
  const form = new FormData();
  form.append('plan', file);
  if (name) form.append('name', name);
  if (repoUrl) form.append('repo_url', repoUrl);

  const res = await fetch(`${API_BASE}/v1/projects`, {
    method: 'POST',
    body: form,
  });

  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || `HTTP ${res.status}`);
  }

  return res.json() as Promise<CreateProjectResponse>;
}

export async function cancelProject(id: string): Promise<void> {
  await request(`/v1/projects/${id}/cancel`, { method: 'POST' });
}

export function subscribeProjectStream(
  id: string,
  onUpdate: (data: unknown) => void,
  onError?: (err: Event) => void,
): () => void {
  const source = new EventSource(`${API_BASE}/v1/projects/${id}/stream`);
  source.addEventListener('status', (e) => {
    try {
      onUpdate(JSON.parse(e.data));
    } catch {
      onUpdate(e.data);
    }
  });
  source.onerror = (e) => {
    onError?.(e);
    source.close();
  };
  return () => source.close();
}
