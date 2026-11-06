/** @param {any} node */
function walk(node) {
  const children = node.children ?? [];
  for (let i = 0; i < children.length; i++) {
    const child = children[i];
    if (child.type === 'element' && child.tagName === 'table') {
      children[i] = {
        type: 'element',
        tagName: 'div',
        properties: { className: ['table-scroll'] },
        children: [child]
      };
    } else {
      walk(child);
    }
  }
}

export function tableScroll() {
  return (/** @type {any} */ tree) => walk(tree);
}
