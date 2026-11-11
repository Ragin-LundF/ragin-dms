import {useEffect, useRef} from "react";
import {commands} from "../../ipc";
import {logStore} from "../../session/session";
import {Button} from "../../shared/components";
import {useStore} from "../../shared/store";

/** What the app did in this session, newest at the bottom; follows new lines as they come. */
export function LogPanel() {
  const entries = useStore(logStore);
  const list = useRef<HTMLDivElement>(null);
  useEffect(() => {
    list.current?.scrollTo({ top: list.current.scrollHeight });
  }, [entries]);
  return (
    <section className="log-panel">
      <div className="divider" />
      <div className="row" style={{ padding: "0 var(--space-l)", justifyContent: "space-between" }}>
        <span className="label-large">Log of this session</span>
        <Button
          variant="text"
          disabled={entries.length === 0}
          onClick={() => {
            void commands.clearLog();
            logStore.set([]);
          }}
        >
          Clear
        </Button>
      </div>
      <div ref={list} className="scroll selectable" style={{ flex: 1, padding: "0 var(--space-l)" }}>
        {entries.map((entry, index) => (
          <div key={index} className={`body-small mono nowrap ${entry.isError ? "error-text" : "muted"}`}>
            {clockTime(entry.at)}  {entry.text}
          </div>
        ))}
      </div>
    </section>
  );
}

/** `HH:mm:ss` in local time, as Kotlin's pattern wrote it in every locale. */
function clockTime(at: string): string {
  const date = new Date(at);
  return [date.getHours(), date.getMinutes(), date.getSeconds()].map((part) => String(part).padStart(2, "0")).join(":");
}
