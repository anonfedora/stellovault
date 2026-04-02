import React, { useState, useRef, useCallback } from 'react';

export interface TooltipContent {
  componentName: string;
  weight: number;
  description: string;
  score: number;
}

interface TooltipProps {
  content: TooltipContent;
  children: React.ReactNode;
}

const HIDE_DELAY_MS = 150;

const Tooltip: React.FC<TooltipProps> = ({ content, children }) => {
  const [visible, setVisible] = useState(false);
  const hideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearHideTimer = useCallback(() => {
    if (hideTimerRef.current !== null) {
      clearTimeout(hideTimerRef.current);
      hideTimerRef.current = null;
    }
  }, []);

  const scheduleHide = useCallback(() => {
    clearHideTimer();
    hideTimerRef.current = setTimeout(() => setVisible(false), HIDE_DELAY_MS);
  }, [clearHideTimer]);

  const show = useCallback(() => {
    clearHideTimer();
    setVisible(true);
  }, [clearHideTimer]);

  const handleTriggerKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      setVisible((v) => !v);
    } else if (e.key === 'Escape') {
      setVisible(false);
    }
  }, []);

  const handleDocumentKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') setVisible(false);
  }, []);

  React.useEffect(() => {
    if (visible) {
      document.addEventListener('keydown', handleDocumentKeyDown);
    }
    return () => document.removeEventListener('keydown', handleDocumentKeyDown);
  }, [visible, handleDocumentKeyDown]);

  return (
    <div className="relative inline-flex items-center">
      {/* Trigger wrapper */}
      <div
        onMouseEnter={show}
        onMouseLeave={scheduleHide}
        onKeyDown={handleTriggerKeyDown}
        tabIndex={0}
        role="button"
        aria-expanded={visible}
        aria-haspopup="true"
        className="inline-flex items-center cursor-pointer focus:outline-none"
      >
        {children}
      </div>

      {/* Tooltip body */}
      {visible && (
        <div
          role="tooltip"
          onMouseEnter={clearHideTimer}
          onMouseLeave={scheduleHide}
          className="absolute top-full left-1/2 -translate-x-1/2 mt-2 z-50 w-64 rounded-lg bg-gray-900 text-white text-xs shadow-xl p-3 space-y-1.5"
        >
          {/* Arrow */}
          <div className="absolute -top-1.5 left-1/2 -translate-x-1/2 w-3 h-3 bg-gray-900 rotate-45" />

          <p className="font-bold text-sm leading-tight">{content.componentName}</p>
          <p className="text-gray-300">Weight: {Math.round(content.weight * 100)}%</p>
          <p className="text-gray-200 leading-snug">{content.description}</p>
          <p className="text-gray-300">Score: <span className="font-semibold text-white">{content.score}</span> / 1000</p>
        </div>
      )}
    </div>
  );
};

export default Tooltip;
