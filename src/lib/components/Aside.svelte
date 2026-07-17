<script lang="ts">
  type Variant = 'note' | 'tip' | 'caution' | 'danger';
  type Props = {
    type?: Variant;
    title?: string;
    children: import('svelte').Snippet;
  };
  let { type = 'note', title, children }: Props = $props();

  const styles: Record<Variant, { border: string; bg: string; text: string; label: string }> = {
    note: {
      border: 'border-[var(--color-plum)]',
      bg: 'bg-purple-50 dark:bg-purple-950/30',
      text: 'text-purple-950 dark:text-purple-100',
      label: 'Note'
    },
    tip: {
      border: 'border-emerald-400',
      bg: 'bg-emerald-50 dark:bg-emerald-950/30',
      text: 'text-emerald-900 dark:text-emerald-100',
      label: 'Tip'
    },
    caution: {
      border: 'border-amber-400',
      bg: 'bg-amber-50 dark:bg-amber-950/30',
      text: 'text-amber-900 dark:text-amber-100',
      label: 'Caution'
    },
    danger: {
      border: 'border-red-400',
      bg: 'bg-red-50 dark:bg-red-950/30',
      text: 'text-red-900 dark:text-red-100',
      label: 'Danger'
    }
  };

  let s = $derived(styles[type]);
</script>

<aside class="my-6 rounded-md border-l-4 {s.border} {s.bg} {s.text} px-4 py-3">
  <div class="mb-1 font-mono text-xs font-medium uppercase tracking-[0.14em]">
    {title ?? s.label}
  </div>
  <div class="text-sm leading-relaxed">{@render children()}</div>
</aside>
