/** Svelte action: append a `#` permalink anchor to each h2/h3/h4 that has an id. */
export function headingAnchors(node: HTMLElement) {
  enhance(node);

  const observer = new MutationObserver(() => enhance(node));
  observer.observe(node, { childList: true, subtree: true });

  return {
    destroy() {
      observer.disconnect();
    }
  };
}

function enhance(root: HTMLElement) {
  const headings = root.querySelectorAll<HTMLHeadingElement>(
    'h2[id], h3[id], h4[id]'
  );
  for (const h of headings) {
    if (h.querySelector(':scope > .heading-anchor')) continue;
    const a = document.createElement('a');
    a.href = `#${h.id}`;
    a.className = 'heading-anchor';
    a.setAttribute('aria-label', 'Permalink to this section');
    a.textContent = '#';
    h.append(a);
  }
}
