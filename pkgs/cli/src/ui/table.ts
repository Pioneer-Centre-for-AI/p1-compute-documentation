export type TableOptions = {
  indent?: string;
  gap?: string;
  head?: string[];
  headStyle?: (line: string) => string;
  style?: (text: string, row: number, col: number) => string;
};

export function table(rows: string[][], opts: TableOptions = {}): string[] {
  const { indent = '  ', gap = '  ', head, headStyle = (l) => l, style = (t) => t } = opts;
  const all = head ? [head, ...rows] : rows;
  const columns = Math.max(0, ...all.map((r) => r.length));
  const widths = Array.from({ length: columns }, (_, c) =>
    Math.max(0, ...all.map((r) => Bun.stringWidth(r[c] ?? '')))
  );
  const pad = (text: string, c: number) =>
    ' '.repeat(Math.max(0, widths[c]! - Bun.stringWidth(text)));

  const lines: string[] = [];
  if (head)
    lines.push(
      indent +
        headStyle(
          head
            .map((h, c) => h + pad(h, c))
            .join(gap)
            .trimEnd()
        )
    );
  rows.forEach((row, r) => {
    const cells = row.map((text, c) => {
      const last = c === row.length - 1;
      return style(text, r, c) + (last ? '' : pad(text, c));
    });
    lines.push(indent + cells.join(gap).trimEnd());
  });
  return lines;
}
