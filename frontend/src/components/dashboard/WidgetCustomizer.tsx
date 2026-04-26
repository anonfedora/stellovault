"use client";

import { useEffect, useRef, useState } from "react";
import { Settings2 } from "lucide-react";

export interface WidgetDefinition {
  id: string;
  label: string;
  description?: string;
}

interface WidgetCustomizerProps {
  widgets: WidgetDefinition[];
  visibleWidgets: Record<string, boolean>;
  onChange: (next: Record<string, boolean>) => void;
}

export const WidgetCustomizer = ({
  widgets,
  visibleWidgets,
  onChange,
}: WidgetCustomizerProps) => {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const toggle = (id: string) => {
    onChange({ ...visibleWidgets, [id]: !visibleWidgets[id] });
  };

  return (
    <div className="relative" ref={containerRef}>
      <button
        type="button"
        onClick={() => setOpen((prev) => !prev)}
        className="inline-flex items-center gap-2 rounded-md border border-gray-200 dark:border-gray-700 px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
        aria-expanded={open}
        aria-haspopup="true"
      >
        <Settings2 className="h-4 w-4" />
        Customise
      </button>

      {open && (
        <div
          className="absolute right-0 mt-2 w-72 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg z-20 p-3"
          role="menu"
        >
          <p className="text-xs font-semibold text-gray-700 dark:text-gray-200 uppercase tracking-wide mb-2">
            Visible widgets
          </p>
          <ul className="space-y-1.5">
            {widgets.map((widget) => {
              const checked = visibleWidgets[widget.id] !== false;
              return (
                <li key={widget.id}>
                  <label className="flex items-start gap-2 cursor-pointer rounded-md p-2 hover:bg-gray-50 dark:hover:bg-gray-700">
                    <input
                      type="checkbox"
                      className="mt-0.5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                      checked={checked}
                      onChange={() => toggle(widget.id)}
                    />
                    <span>
                      <span className="block text-sm font-medium text-gray-900 dark:text-gray-100">
                        {widget.label}
                      </span>
                      {widget.description && (
                        <span className="block text-xs text-gray-500 dark:text-gray-400">
                          {widget.description}
                        </span>
                      )}
                    </span>
                  </label>
                </li>
              );
            })}
          </ul>
        </div>
      )}
    </div>
  );
};
