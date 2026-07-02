import { useCallback, useEffect, useState } from 'react';
import { getProject } from '../api/client';
import type { ProjectDetail } from '../types';

export function useProject(id: string | undefined, pollMs = 3000) {
  const [project, setProject] = useState<ProjectDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!id) return;
    try {
      const data = await getProject(id);
      setProject(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load project');
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    if (!id) return;
    refresh();
    const timer = setInterval(refresh, pollMs);
    return () => clearInterval(timer);
  }, [id, pollMs, refresh]);

  return { project, loading, error, refresh };
}
