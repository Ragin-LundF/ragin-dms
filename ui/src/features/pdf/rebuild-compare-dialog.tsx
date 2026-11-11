import AUTO_FIX_HIGH from "@material-design-icons/svg/round/auto_fix_high.svg?raw";
import KEYBOARD_ARROW_LEFT from "@material-design-icons/svg/round/keyboard_arrow_left.svg?raw";
import KEYBOARD_ARROW_RIGHT from "@material-design-icons/svg/round/keyboard_arrow_right.svg?raw";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { convertFileSrc } from "@tauri-apps/api/core";
import { commands, pageUrl, type RebuildChoiceDto, type RebuildCompareDto_Serialize, type RegionDto, type RegionKindDto } from "../../ipc";
import { LIBRARY_QUERY, reportError } from "../../session/session";
import { Button, Dialog, Icon, IconButton, Spinner, dialogs } from "../../shared/components";
import { formatFileSize } from "../../shared/format/sizes";

type CompareView = "sideBySide" | "original" | "rebuilt";

/** About 100 dpi for an A4 page: the routes take it as the long side, as Kotlin rendered it. */
const PICTURE_SIDE = 1170;

/** Blue, orange and green: the first three chart colours, safe for colour blindness. */
const REGION_STYLES: Record<RegionKindDto, { color: string; label: string }> = {
  text: { color: "var(--chart-3)", label: "real text" },
  keptLine: { color: "var(--chart-2)", label: "line kept as image" },
  image: { color: "var(--chart-1)", label: "picture of the scan" },
};

/** A page of the rebuilt file, served by the shell's `dms://…/rebuilt/` route. */
function rebuiltPageUrl(rebuildId: string, page: number, width: number): string {
  return `${convertFileSrc(`rebuilt/${rebuildId}/${page}`, "dms")}?width=${width}`;
}

/** The scan and its rebuild, compared page by page, with the choice of which to keep. */
export function RebuildCompareDialog() {
  const compare = useQuery({ queryKey: [LIBRARY_QUERY, "rebuild-compare"], queryFn: () => commands.rebuildCompare() });
  const state = compare.data;
  if (!state) return null;
  return <RebuildCompare key={state.rebuildId} state={state} />;
}

function RebuildCompare({ state }: { state: RebuildCompareDto_Serialize }) {
  const [page, setPage] = useState(0);
  const [view, setView] = useState<CompareView>("sideBySide");
  const [overlay, setOverlay] = useState(false);
  const rebuilt = state.pages[page];
  const pages = state.pages.length;
  if (!rebuilt) return null;
  return (
    // Only the three buttons close it: a stray click outside or Esc must not throw the rebuild away.
    <Dialog
      title={
        <span className="column pdf-dialog-heading">
          <Icon svg={AUTO_FIX_HIGH} size={24} />
          Compare with the rebuilt PDF
        </span>
      }
      width="min(90vw, 1000px)" className="pdf-wide-dialog"
      onDismiss={stayOpen}
      buttons={
        <>
          <Button variant="text" onClick={() => void choose(state, "keepOriginal")}>Keep original</Button>
          <Button variant="outlined" onClick={() => void choose(state, "keepBoth")}>Keep both</Button>
          <Button variant="filled" onClick={() => void choose(state, "replaceWithRebuild")}>Use rebuilt PDF</Button>
        </>
      }
    >
      <div className="column gap-s">
        <span className="body-medium">
          {`${rebuilt.linesAsText} of ${rebuilt.linesTotal} lines on this page are real text · Original ${formatFileSize(state.originalSize ?? 0)} · rebuilt ${formatFileSize(state.rebuiltSize ?? 0)}`}
        </span>
        {rebuilt.kept && <span className="body-small muted">{rebuilt.kept}</span>}
        <div className="row gap-s">
          <IconButton icon={KEYBOARD_ARROW_LEFT} label="Previous page" disabled={page === 0} onClick={() => setPage(page - 1)} />
          <span className="body-medium nowrap">{`Page ${page + 1} of ${pages}`}</span>
          <IconButton icon={KEYBOARD_ARROW_RIGHT} label="Next page" disabled={page >= pages - 1} onClick={() => setPage(page + 1)} />
          <Button onClick={() => setView(view === "rebuilt" ? "original" : "rebuilt")}>Flip</Button>
          {view !== "sideBySide" && <Button variant="text" onClick={() => setView("sideBySide")}>Side by side</Button>}
          <label className="row">
            <input type="checkbox" className="checkbox" checked={overlay} onChange={() => setOverlay(!overlay)} />
            <span className="body-medium">Mark text, kept lines and images</span>
          </label>
        </div>
        {overlay && (
          <div className="row gap-l">
            {(["text", "keptLine", "image"] as const).map((kind) => (
              <span key={kind} className="body-small" style={{ color: REGION_STYLES[kind].color }}>■ {REGION_STYLES[kind].label}</span>
            ))}
          </div>
        )}
        <div className="row gap-m pdf-compare-panes">
          {view !== "rebuilt" && <Pane label="Original" url={pageUrl(state.documentId, page, PICTURE_SIDE)} regions={[]} />}
          {view !== "original" && <Pane label="Rebuilt" url={rebuiltPageUrl(state.rebuildId, page, PICTURE_SIDE)} regions={overlay ? rebuilt.regions : []} />}
        </div>
      </div>
    </Dialog>
  );
}

/** Only the three choices close the comparison; stable, since the dialog refocuses when `onDismiss` changes. */
function stayOpen() {}

async function choose(state: RebuildCompareDto_Serialize, choice: RebuildChoiceDto) {
  if (choice === "replaceWithRebuild") {
    const confirmed = await dialogs.confirm({
      title: "Use the rebuilt PDF",
      message: "The scan moves to the trash, and the rebuilt PDF takes its place in every folder.",
      affected: [state.path],
      confirmLabel: "Use rebuilt PDF",
    });
    if (!confirmed) return;
  }
  await commands.chooseRebuild(choice).catch(reportError);
}

/** One page picture, a spinner until it is there, and the outlines of the regions over it. */
function Pane({ label, url, regions }: { label: string; url: string; regions: RegionDto[] }) {
  const [loaded, setLoaded] = useState<string | null>(null);
  return (
    <div className="column pdf-compare-pane">
      <span className="label-large">{label}</span>
      <div className="pdf-compare-picture">
        {loaded !== url && <Spinner />}
        <div className="pdf-compare-frame" style={{ display: loaded === url ? undefined : "none" }}>
          <img src={url} alt={`${label} page`} draggable={false} onLoad={() => setLoaded(url)} />
          {regions.length > 0 && (
            <svg viewBox="0 0 1000 1000" preserveAspectRatio="none" aria-hidden>
              {regions.map((region, index) => (
                <rect
                  key={index}
                  x={region.left}
                  y={region.top}
                  width={region.right - region.left}
                  height={region.bottom - region.top}
                  fill="none"
                  stroke={REGION_STYLES[region.kind].color}
                  strokeWidth={1.5}
                  vectorEffect="non-scaling-stroke"
                />
              ))}
            </svg>
          )}
        </div>
      </div>
    </div>
  );
}
