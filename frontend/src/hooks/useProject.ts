import { useCallback, useEffect, useRef, useState } from 'react';
import { getProject, subscribeProjectStream } from '../api/client';
import type { ProjectDetail } from '../types';

const TERMINAL = new Set(['completed', 'failed', 'cancelled']);

export function useProject(id: string | undefined, pollMs = 3000) {
  const [project, setProject] = useState<ProjectDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const streamActive = useRef(false);

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

    setLoading(true);
    refresh();

    if (typeof EventSource !== 'undefined') {
      streamActive.current = true;
      const cleanup = subscribeProjectStream(
        id,
        (data) => {
          const detail = data as ProjectDetail;
          if (detail?.id) {
            setProject(detail);
            setError(null);
            setLoading(false);
            if (TERMINAL.has(detail.state)) {
              streamActive.current = false;
            }
          }
        },
        () => {
          streamActive.current = false;
        },
      );
      return () => {
        cleanup();
        streamActive.current = false;
      };
    }
  }, [id, refresh]);

  useEffect(() => {
    if (!id) return;

    const active =
      project?.state === 'running' || project?.state === 'awaiting_input';
    const intervalMs = active ? 2000 : pollMs;

    const timer = setInterval(() => {
      if (!streamActive.current) {
        refresh();
      }
    }, intervalMs);

    return () => clearInterval(timer);
  }, [id, pollMs, project?.state, refresh]);

  return { project, loading, error, refresh };
}
