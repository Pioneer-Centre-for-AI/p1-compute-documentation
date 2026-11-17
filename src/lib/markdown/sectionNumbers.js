const NUMBER = /^(\d+(?:\.\d+)*)\.\s+/;

/** @param {any} node */
function walk(node) {
  for (const child of node.children ?? []) {
    if (child.type === 'element' && (child.tagName === 'h2' || child.tagName === 'h3')) {
      const first = child.children?.[0];
      const match = first?.type === 'text' ? NUMBER.exec(first.value) : null;
      if (match) {
        first.value = ` ${first.value.slice(match[0].length)}`;
        child.children.unshift({
          type: 'element',
          tagName: 'span',
          properties: { className: ['section-number'] },
          children: [{ type: 'text', value: match[1] }]
        });
      }
    } else {
      walk(child);
    }
  }
}

export function sectionNumbers() {
  return (/** @type {any} */ tree) => walk(tree);
}
