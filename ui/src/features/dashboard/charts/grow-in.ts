import {useEffect, useState} from "react";

/** 0 on the first frame, then `target`; with a 700 ms CSS transition the value grows in (Kotlin `growIn`). */
export function useGrowIn(target: number): number {
  const [shown, setShown] = useState(false);
  useEffect(() => {
    const frame = requestAnimationFrame(() => setShown(true));
    return () => cancelAnimationFrame(frame);
  }, []);
  return shown ? target : 0;
}
