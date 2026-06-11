import { createHighlighter, type Highlighter } from 'shiki';

let highlighterPromise: Promise<Highlighter> | null = null;

export function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: ['github-light', 'github-dark'],
      langs: ['bash', 'text']
    });
  }
  return highlighterPromise;
}

export async function highlight(code: string, lang: string): Promise<string> {
  const hl = await getHighlighter();
  const langs = hl.getLoadedLanguages();
  const safeLang = langs.includes(lang as never) ? lang : 'text';
  return hl.codeToHtml(code, {
    lang: safeLang,
    themes: { light: 'github-light', dark: 'github-dark' },
    defaultColor: false
  });
}
