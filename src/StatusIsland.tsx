import { type MouseEvent, type TouchEvent, useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow, currentMonitor, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import { dashboardSummary, toggleDashboard } from "./api";
import { calculateDockPlacement, type DockEdge } from "./statusIslandDock";
import type { DashboardSummary } from "./types";

const REFRESH_INTERVAL_MS = 5_000;
const SNAP_DELAY_MS = 180;
const HORIZONTAL_SIZE = { width: 240, height: 22 };
const SIDE_SIZE = { width: 40, height: 112 };

const emptySummary: DashboardSummary = {
  today_total_tokens: 0,
  last_hour_tokens: 0,
  last_five_hours_tokens: 0,
  active_session_count: 0,
  input_tokens: 0,
  output_tokens: 0,
};

const numberFormat = new Intl.NumberFormat("en", {
  notation: "compact",
  maximumFractionDigits: 1,
});

function formatTokens(value: number): string {
  return numberFormat.format(value);
}

export function StatusIsland() {
  const [summary, setSummary] = useState<DashboardSummary>(emptySummary);
  const [isConnected, setIsConnected] = useState(true);
  const [dockEdge, setDockEdge] = useState<DockEdge>("top");
  const snapTimerRef = useRef<number | null>(null);
  const isSnappingRef = useRef(false);
  const dragStateRef = useRef<{
    pointerStart: { x: number; y: number };
    windowStart: { x: number; y: number };
    lastPosition: { x: number; y: number };
  } | null>(null);

  useEffect(() => {
    document.documentElement.classList.add("status-window-root");
    document.body.classList.add("status-window");
    void getCurrentWindow().setSize(new PhysicalSize(HORIZONTAL_SIZE.width, HORIZONTAL_SIZE.height));
    return () => {
      document.documentElement.classList.remove("status-window-root");
      document.body.classList.remove("status-window");
    };
  }, []);

  useEffect(() => {
    let isMounted = true;
    let isRequestInFlight = false;

    const loadSummary = async () => {
      if (isRequestInFlight) {
        return;
      }

      isRequestInFlight = true;
      try {
        const nextSummary = await dashboardSummary();
        if (!isMounted) {
          return;
        }

        setSummary(nextSummary);
        setIsConnected(true);
      } catch {
        if (isMounted) {
          setIsConnected(false);
        }
      } finally {
        isRequestInFlight = false;
      }
    };

    void loadSummary();
    const refreshId = window.setInterval(() => {
      void loadSummary();
    }, REFRESH_INTERVAL_MS);

    return () => {
      isMounted = false;
      window.clearInterval(refreshId);
    };
  }, []);

  const snapToNearestEdge = useCallback(async (position: { x: number; y: number }) => {
    if (isSnappingRef.current) {
      return;
    }

    const monitor = await currentMonitor();
    if (!monitor) {
      return;
    }

    const appWindow = getCurrentWindow();
    const currentSize = await appWindow.outerSize();
    const placement = calculateDockPlacement({
      position,
      currentSize,
      workArea: {
        x: monitor.workArea.position.x,
        y: monitor.workArea.position.y,
        width: monitor.workArea.size.width,
        height: monitor.workArea.size.height,
      },
      horizontalSize: HORIZONTAL_SIZE,
      sideSize: SIDE_SIZE,
    });

    isSnappingRef.current = true;
    setDockEdge(placement.edge);
    await appWindow.setSize(new PhysicalSize(placement.size.width, placement.size.height));
    await appWindow.setPosition(new PhysicalPosition(placement.position.x, placement.position.y));
    window.setTimeout(() => {
      isSnappingRef.current = false;
    }, SNAP_DELAY_MS);
  }, []);

  useEffect(() => {
    const appWindow = getCurrentWindow();
    let unlisten: (() => void) | null = null;

    void appWindow.onMoved(({ payload }) => {
      if (isSnappingRef.current) {
        return;
      }

      if (snapTimerRef.current !== null) {
        window.clearTimeout(snapTimerRef.current);
      }

      snapTimerRef.current = window.setTimeout(() => {
        void snapToNearestEdge(payload);
      }, SNAP_DELAY_MS);
    }).then((nextUnlisten) => {
      unlisten = nextUnlisten;
    });

    return () => {
      if (snapTimerRef.current !== null) {
        window.clearTimeout(snapTimerRef.current);
      }
      unlisten?.();
    };
  }, [snapToNearestEdge]);

  const handleStartDrag = useCallback((event: MouseEvent<HTMLElement> | TouchEvent<HTMLElement>) => {
    if ("button" in event && event.button !== 0) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const point =
      "touches" in event
        ? { x: event.touches[0]?.screenX ?? 0, y: event.touches[0]?.screenY ?? 0 }
        : { x: event.screenX, y: event.screenY };

    const appWindow = getCurrentWindow();
    void appWindow.outerPosition().then((windowStart) => {
      dragStateRef.current = {
        pointerStart: point,
        windowStart,
        lastPosition: windowStart,
      };
    });
    void appWindow.startDragging();
  }, []);

  useEffect(() => {
    const moveWindow = (screenX: number, screenY: number) => {
      const dragState = dragStateRef.current;
      if (!dragState) {
        return;
      }

      const nextPosition = {
        x: dragState.windowStart.x + Math.round(screenX - dragState.pointerStart.x),
        y: dragState.windowStart.y + Math.round(screenY - dragState.pointerStart.y),
      };
      dragState.lastPosition = nextPosition;
      void getCurrentWindow().setPosition(new PhysicalPosition(nextPosition.x, nextPosition.y));
    };

    const handleMouseMove = (event: globalThis.MouseEvent) => {
      moveWindow(event.screenX, event.screenY);
    };

    const handleTouchMove = (event: globalThis.TouchEvent) => {
      const touch = event.touches[0];
      if (touch) {
        moveWindow(touch.screenX, touch.screenY);
      }
    };

    const finishDrag = () => {
      const dragState = dragStateRef.current;
      dragStateRef.current = null;
      if (dragState) {
        void snapToNearestEdge(dragState.lastPosition);
      }
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", finishDrag);
    window.addEventListener("touchmove", handleTouchMove, { passive: true });
    window.addEventListener("touchend", finishDrag);
    window.addEventListener("blur", finishDrag);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", finishDrag);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("touchend", finishDrag);
      window.removeEventListener("blur", finishDrag);
    };
  }, [snapToNearestEdge]);

  const handleToggleDashboard = useCallback(() => {
    void toggleDashboard();
  }, []);

  return (
    <main
      className={`status-island status-island-${dockEdge}`}
      aria-label="Codex token status"
      data-tauri-drag-region
      onMouseDown={handleStartDrag}
      onTouchStart={handleStartDrag}
    >
      <div
        className="status-drag-grip"
        role="button"
        tabIndex={0}
        aria-label="Move status island"
        data-tauri-drag-region
        onMouseDown={handleStartDrag}
        onTouchStart={handleStartDrag}
      />
      <button
        className="status-island-content"
        type="button"
        onClick={handleToggleDashboard}
        onMouseDown={(event) => event.stopPropagation()}
        onTouchStart={(event) => event.stopPropagation()}
      >
        <span className={isConnected ? "status-dot is-live" : "status-dot"} aria-hidden="true" />
        <span className="status-metric">
          <span className="status-label">Today</span>
          <strong>{formatTokens(summary.today_total_tokens)}</strong>
        </span>
        <span className="status-metric">
          <span className="status-label">1h</span>
          <strong>{formatTokens(summary.last_hour_tokens)}</strong>
        </span>
        <span className="status-metric">
          <span className="status-label">S</span>
          <strong>{summary.active_session_count}</strong>
        </span>
      </button>
    </main>
  );
}
