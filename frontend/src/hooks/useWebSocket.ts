import { useEffect, useRef, useState } from "react";
import type { RiskScoreBreakdown } from "./useRiskScore";

// ─── Public types ─────────────────────────────────────────────────────────────

export interface ScoreUpdateEvent {
  type: "RiskScoreUpdated";
  wallet_address: string;
  overall_score: number;
  grade: "A" | "B" | "C" | "D" | "F";
  components: RiskScoreBreakdown[];
  calculated_at: string;
}

export interface UseWebSocketOptions {
  walletAddress: string | null;
  onScoreUpdate: (event: ScoreUpdateEvent) => void;
}

export interface UseWebSocketReturn {
  connected: boolean;
  connectionFailed: boolean;
}

// ─── Reconnect delay helper (exported for property-based testing) ─────────────

export function computeReconnectDelay(attempt: number): number {
  return Math.min(Math.pow(2, attempt - 1) * 1000, 30000);
}

// ─── Hook ─────────────────────────────────────────────────────────────────────

const MAX_FAILURES = 3;

export function useWebSocket(options: UseWebSocketOptions): UseWebSocketReturn {
  const { walletAddress, onScoreUpdate } = options;

  const [connected, setConnected] = useState(false);
  const [connectionFailed, setConnectionFailed] = useState(false);

  // Stable ref to the latest callback so we don't need to re-open the socket
  // when the parent re-renders with a new function reference.
  const onScoreUpdateRef = useRef(onScoreUpdate);
  useEffect(() => {
    onScoreUpdateRef.current = onScoreUpdate;
  }, [onScoreUpdate]);

  useEffect(() => {
    if (!walletAddress) {
      return;
    }

    let ws: WebSocket | null = null;
    let consecutiveFailures = 0;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let stopped = false;

    function clearTimer() {
      if (reconnectTimer !== null) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
    }

    function connect() {
      if (stopped) return;

      ws = new WebSocket("/ws");

      ws.onopen = () => {
        if (stopped) {
          ws?.close();
          return;
        }
        consecutiveFailures = 0;
        setConnected(true);
        setConnectionFailed(false);
      };

      ws.onmessage = (event: MessageEvent) => {
        if (stopped) return;
        try {
          const data = JSON.parse(event.data as string);
          if (data && data.type === "RiskScoreUpdated") {
            onScoreUpdateRef.current(data as ScoreUpdateEvent);
          }
        } catch {
          // Ignore malformed messages
        }
      };

      ws.onclose = () => {
        if (stopped) return;
        setConnected(false);
        consecutiveFailures += 1;

        if (consecutiveFailures >= MAX_FAILURES) {
          setConnectionFailed(true);
          return; // Stop reconnecting
        }

        const delay = computeReconnectDelay(consecutiveFailures);
        reconnectTimer = setTimeout(connect, delay);
      };

      ws.onerror = () => {
        if (stopped) return;
        setConnected(false);
        // onclose will fire after onerror, so we handle reconnect there
      };
    }

    connect();

    return () => {
      stopped = true;
      clearTimer();
      if (ws) {
        ws.onopen = null;
        ws.onmessage = null;
        ws.onclose = null;
        ws.onerror = null;
        ws.close();
        ws = null;
      }
      setConnected(false);
    };
  }, [walletAddress]);

  const effectiveConnected = walletAddress ? connected : false;
  const effectiveConnectionFailed = walletAddress ? connectionFailed : false;

  return {
    connected: effectiveConnected,
    connectionFailed: effectiveConnectionFailed,
  };
}
