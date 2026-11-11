import {type CSSProperties, type ReactNode, useState} from "react";
import {thumbnailUrl} from "../../../ipc";

/**
 * The picture of the file at `path` in library folder `folderId`, fitted into its box (Kotlin `PageThumbnail`).
 * Until the picture is there, and for files without one, `fallback` is shown instead.
 */
export function PageThumbnail({ folderId, path, fallback, style, className }: { folderId: string; path: string; fallback: ReactNode; style?: CSSProperties; className?: string }) {
  const url = thumbnailUrl(folderId, path);
  const [state, setState] = useState<{ url: string; loaded: boolean; failed: boolean }>({ url, loaded: false, failed: false });
  const current = state.url === url ? state : { url, loaded: false, failed: false };
  const fileName = path.split("/").pop() ?? path;
  return (
    <>
      {!current.loaded && fallback}
      {!current.failed && (
        <img
          src={url}
          alt={`Preview of ${fileName}`}
          draggable={false}
          className={className}
          style={{ objectFit: "contain", display: current.loaded ? undefined : "none", ...style }}
          onLoad={() => setState({ url, loaded: true, failed: false })}
          onError={() => setState({ url, loaded: false, failed: true })}
        />
      )}
    </>
  );
}

/** A clickable 64 × 88 thumbnail with a hairline border (Kotlin `FileThumbnail`). */
export function FileThumbnail({ folderId, path, onClick, fallback }: { folderId: string; path: string; onClick: () => void; fallback: ReactNode }) {
  return (
    <button type="button" className="file-thumbnail" onClick={onClick} aria-label={`Open ${path.split("/").pop() ?? path}`}>
      <PageThumbnail folderId={folderId} path={path} fallback={fallback} style={{ width: "100%", height: "100%" }} />
    </button>
  );
}
