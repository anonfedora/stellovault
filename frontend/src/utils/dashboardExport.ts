// Helpers for exporting dashboard data as CSV or JSON.
// Pure functions so they can be unit-tested without a DOM.

export type ExportFormat = "csv" | "json";

export interface ExportableRow {
  [key: string]: string | number | boolean | null | undefined;
}

/**
 * Serialise a list of rows into a CSV string. Values containing commas,
 * quotes, or newlines are quoted and inner quotes are escaped per RFC 4180.
 */
export function rowsToCsv(rows: ExportableRow[]): string {
  if (rows.length === 0) return "";

  const headers = Array.from(
    rows.reduce<Set<string>>((acc, row) => {
      Object.keys(row).forEach((key) => acc.add(key));
      return acc;
    }, new Set<string>()),
  );

  const escape = (value: ExportableRow[string]): string => {
    if (value === null || value === undefined) return "";
    let str = String(value);
    // Mitigate CSV/formula injection (Excel, Sheets, LibreOffice) by prefixing
    // values that start with a formula-trigger char with a single quote.
    if (/^[=+\-@\t\r]/.test(str)) {
      str = `'${str}`;
    }
    if (/[",\n\r]/.test(str)) {
      return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
  };

  const lines = [headers.join(",")];
  for (const row of rows) {
    lines.push(headers.map((h) => escape(row[h])).join(","));
  }
  // RFC 4180 specifies CRLF as the record separator.
  return lines.join("\r\n");
}

/**
 * Serialise a payload into a string for the given format.
 * For JSON we always use 2-space indentation for readability.
 */
export function serialiseExport(
  data: ExportableRow[],
  format: ExportFormat,
): string {
  if (format === "csv") return rowsToCsv(data);
  return JSON.stringify(data, null, 2);
}

/**
 * Return a filesystem-safe filename for an export, using the provided slug
 * and an ISO-like timestamp.
 */
export function buildExportFilename(slug: string, format: ExportFormat): string {
  const safe = slug.replace(/[^a-z0-9-_]+/gi, "-").toLowerCase();
  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  return `${safe}-${stamp}.${format}`;
}

/**
 * Trigger a browser download for the given content. No-op when called outside
 * of a browser environment (e.g. during SSR or in unit tests).
 */
export function downloadFile(
  filename: string,
  content: string,
  mime: string,
): void {
  if (typeof window === "undefined" || typeof document === "undefined") return;
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  // Delay revoking the URL so the browser has time to start the download
  // (notably Safari iOS, which can race with synchronous revocation).
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

/**
 * Convenience helper that serialises and downloads the report in one call.
 */
export function exportDashboardReport(
  slug: string,
  rows: ExportableRow[],
  format: ExportFormat,
): { filename: string; content: string } {
  const content = serialiseExport(rows, format);
  const filename = buildExportFilename(slug, format);
  const mime = format === "csv" ? "text/csv;charset=utf-8" : "application/json";
  downloadFile(filename, content, mime);
  return { filename, content };
}
