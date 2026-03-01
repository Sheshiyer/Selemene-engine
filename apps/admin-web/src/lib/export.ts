export function toCsv<T extends object>(
  rows: T[],
  columns: Array<{ key: keyof T; header: string }>
): string {
  const escapeCell = (value: unknown): string => {
    const raw = value == null ? "" : typeof value === "string" ? value : JSON.stringify(value);
    const escaped = raw.replaceAll('"', '""');
    return `"${escaped}"`;
  };

  const headerLine = columns.map((column) => escapeCell(column.header)).join(",");
  const lines = rows.map((row) =>
    columns.map((column) => escapeCell(row[column.key])).join(",")
  );

  return [headerLine, ...lines].join("\n");
}

export function downloadFile(filename: string, content: string, type: string): void {
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

export function exportJson(filename: string, data: unknown): void {
  downloadFile(filename, JSON.stringify(data, null, 2), "application/json;charset=utf-8");
}

export function exportCsv<T extends object>(
  filename: string,
  rows: T[],
  columns: Array<{ key: keyof T; header: string }>
): void {
  downloadFile(filename, toCsv(rows, columns), "text/csv;charset=utf-8");
}

export async function copyToClipboard(value: string): Promise<void> {
  await navigator.clipboard.writeText(value);
}
