import { STAGE_META, type StageId, type StageState } from '../types';
import {
  CheckCircle2,
  Circle,
  Loader2,
  MinusCircle,
  XCircle,
} from 'lucide-react';

const statusIcon: Record<StageState, React.ReactNode> = {
  queued: <Circle size={16} className="muted" />,
  running: <Loader2 size={16} className="spin accent" />,
  completed: <CheckCircle2 size={16} className="success" />,
  failed: <XCircle size={16} className="error" />,
  skipped: <MinusCircle size={16} className="muted" />,
};

export function PipelineStages({
  stages,
}: {
  stages: { stage: StageId; status: StageState }[];
}) {
  return (
    <div className="pipeline-grid">
      {stages.map((s, idx) => {
        const meta = STAGE_META[s.stage];
        return (
          <div
            key={s.stage}
            className={`stage-card ${s.status} ${idx < stages.findIndex((x) => x.status === 'running') ? 'past' : ''}`}
          >
            <div className="stage-card-top">
              {statusIcon[s.status]}
              <span className="stage-index">{idx + 1}</span>
            </div>
            <h4>{meta.label}</h4>
            <p>{meta.description}</p>
            {meta.model && <span className="model-tag">{meta.model}</span>}
            <span className={`stage-status ${s.status}`}>{s.status}</span>
          </div>
        );
      })}
    </div>
  );
}
