import {useEffect} from "react";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import {messageStore} from "../../session/session";
import {IconButton} from "../../shared/components";
import {useStore} from "../../shared/store";

const SHORT_MS = 4_000;
const LONG_MS = 10_000;

/** Shows the oldest waiting message; errors stay longer. Each can be dismissed. */
export function Snackbar() {
  const queue = useStore(messageStore);
  const current = queue[0];
  useEffect(() => {
    if (!current) return;
    const timer = window.setTimeout(dismiss, current.isError ? LONG_MS : SHORT_MS);
    return () => window.clearTimeout(timer);
  }, [current]);
  if (!current) return null;
  return (
    <div className="snackbar-host">
      <div className="snackbar body-medium" role="status">
        <span className="grow">{current.text}</span>
        <IconButton icon={CLOSE} label="Dismiss" onClick={dismiss} />
      </div>
    </div>
  );
}

function dismiss() {
  messageStore.set((queue) => queue.slice(1));
}
