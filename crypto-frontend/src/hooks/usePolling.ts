import { useEffect, useRef } from "react";

export function usePolling<T>(
  callback: () => Promise<T>,
  onResult: (data: T) => void,
  intervalMs: number,
  enabled = true
) {
  const savedCallback = useRef(callback);
  const savedOnResult = useRef(onResult);

  useEffect(() => {
    savedCallback.current = callback;
    savedOnResult.current = onResult;
  }, [callback, onResult]);

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;

    const tick = async () => {
      if (cancelled) return;
      try {
        const data = await savedCallback.current();
        if (!cancelled) savedOnResult.current(data);
      } catch {
        // errors handled by caller
      }
    };

    tick();
    const id = setInterval(tick, intervalMs);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [intervalMs, enabled]);
}
