<script lang="ts">
  import { onMount } from 'svelte';
  import { afterNavigate } from '$app/navigation';

  type TocChild = { id: string; label: string };
  type TocEntry = { id: string; label: string; children?: TocChild[] };

  let entries = $state<TocEntry[]>([]);
  let activeId = $state<string | null>(null);
  let observer: IntersectionObserver | null = null;

  // Heading text without the '#' permalink appended by the headingAnchors action.
  function labelOf(h: HTMLElement): string {
    return Array.from(h.childNodes)
      .filter((n) => !(n instanceof Element && n.classList.contains('heading-anchor')))
      .map((n) => n.textContent ?? '')
      .join('')
      .trim();
  }

  function scan() {
    const root = document.querySelector('article.prose');
    const headings = root
      ? Array.from(root.querySelectorAll<HTMLHeadingElement>('h2[id], h3[id]'))
      : [];

    const tree: TocEntry[] = [];
    for (const h of headings) {
      if (h.tagName === 'H2') {
        tree.push({ id: h.id, label: labelOf(h) });
      } else if (tree.length > 0) {
        (tree[tree.length - 1].children ??= []).push({ id: h.id, label: labelOf(h) });
      }
    }
    entries = tree;
    observe(headings);
  }

  function observe(targets: HTMLElement[]) {
    observer?.disconnect();
    if (targets.length === 0) {
      activeId = null;
      return;
    }

    const ids = targets.map((t) => t.id);
    const visible = new Map<string, number>();

    observer = new IntersectionObserver(
      (observed) => {
        for (const entry of observed) {
          if (entry.isIntersecting) {
            visible.set(entry.target.id, entry.intersectionRatio);
          } else {
            visible.delete(entry.target.id);
          }
        }
        let next: string | null = null;
        for (const id of ids) {
          if (visible.has(id)) {
            next = id;
            break;
          }
        }
        if (next) activeId = next;
      },
      { rootMargin: '-80px 0px -60% 0px', threshold: [0, 1] }
    );

    for (const t of targets) observer.observe(t);
    activeId = targets[0].id;
  }

  onMount(() => {
    scan();
    return () => observer?.disconnect();
  });

  afterNavigate(scan);
</script>

{#if entries.length > 0}
  <nav class="text-sm" aria-label="On this page">
    <div class="mb-3 text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
      On this page
    </div>
    <ul class="space-y-1">
      {#each entries as entry}
        <li>
          <a
            href={`#${entry.id}`}
            class="block rounded px-2 py-1 no-underline {activeId === entry.id ? 'bg-slate-100 font-semibold text-slate-900 dark:bg-slate-800 dark:text-slate-50' : 'text-slate-600 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100'}"
          >
            {entry.label}
          </a>
          {#if entry.children}
            <ul class="ml-3 mt-0.5 space-y-0.5 border-l border-slate-200 dark:border-slate-700">
              {#each entry.children as child}
                <li>
                  <a
                    href={`#${child.id}`}
                    class="block rounded px-2 py-0.5 text-[13px] no-underline {activeId === child.id ? 'font-semibold text-slate-900 dark:text-slate-50' : 'text-slate-500 hover:text-slate-900 dark:text-slate-500 dark:hover:text-slate-100'}"
                  >
                    {child.label}
                  </a>
                </li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  </nav>
{/if}
