import { useEffect, useRef, useState } from 'react';
import { Check, Copy, Image as ImageIcon, Loader2, Upload } from 'lucide-react';
import { listImages, uploadImage } from '../api/client';
import type { HostedImage } from '../types';

const ACCEPT = 'image/png,image/jpeg,image/gif,image/webp,image/bmp,image/svg+xml';

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function ImageHostingPage() {
  const fileRef = useRef<HTMLInputElement>(null);
  const [images, setImages] = useState<HostedImage[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);

  const refresh = () => {
    setLoading(true);
    listImages()
      .then(setImages)
      .catch((err) => setError(err instanceof Error ? err.message : '목록을 불러오지 못했습니다.'))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    refresh();
  }, []);

  const doUpload = async (file: File) => {
    if (!file.type.startsWith('image/')) {
      setError('이미지 파일만 업로드할 수 있습니다.');
      return;
    }
    setUploading(true);
    setError(null);
    try {
      await uploadImage(file);
      refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : '업로드 실패');
    } finally {
      setUploading(false);
    }
  };

  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const dropped = e.dataTransfer.files[0];
    if (dropped) doUpload(dropped);
  };

  const copyLink = (url: string) => {
    navigator.clipboard?.writeText(url).then(() => {
      setCopied(url);
      setTimeout(() => setCopied(null), 1500);
    });
  };

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h2>이미지 호스팅</h2>
          <p>이미지를 업로드하면 바로 공유·임베드 가능한 URL을 받을 수 있습니다.</p>
        </div>
      </header>

      <div className="card upload-form">
        <h3 className="section-title">
          <ImageIcon size={18} />
          이미지 업로드
        </h3>
        <div
          className={`dropzone ${dragOver ? 'drag-over' : ''}`}
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
            accept={ACCEPT}
            hidden
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) doUpload(f);
              e.target.value = '';
            }}
          />
          {uploading ? (
            <>
              <Loader2 size={40} className="spin accent" />
              <strong>업로드 중...</strong>
            </>
          ) : (
            <>
              <Upload size={40} className="muted" />
              <strong>이미지를 드래그하거나 클릭하여 업로드</strong>
              <span className="muted">PNG · JPG · GIF · WEBP · BMP · SVG</span>
            </>
          )}
        </div>

        {error && <div className="alert error">{error}</div>}
      </div>

      <section className="card">
        <h3 className="section-title">업로드된 이미지 ({images.length})</h3>
        {loading ? (
          <p className="muted">불러오는 중...</p>
        ) : images.length === 0 ? (
          <p className="muted">아직 업로드된 이미지가 없습니다.</p>
        ) : (
          <div className="image-grid">
            {images.map((img) => (
              <div key={img.filename} className="image-card">
                <div className="image-preview">
                  <img src={img.url} alt={img.filename} loading="lazy" />
                </div>
                <div className="image-meta">
                  <span className="muted">{formatSize(img.size)}</span>
                  <button
                    type="button"
                    className="btn ghost small"
                    onClick={() => copyLink(img.url)}
                  >
                    {copied === img.url ? (
                      <>
                        <Check size={14} /> 복사됨
                      </>
                    ) : (
                      <>
                        <Copy size={14} /> 링크 복사
                      </>
                    )}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
