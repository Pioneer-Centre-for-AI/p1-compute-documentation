<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';

  type Variant = 'primary' | 'secondary' | 'ghost';

  type Props = {
    variant?: Variant;
    href?: string;
    children: Snippet;
  } & (Omit<HTMLAnchorAttributes, 'children'> & Omit<HTMLButtonAttributes, 'children'>);

  let { variant = 'primary', href, children, ...rest }: Props = $props();

  const base =
    'inline-flex items-center justify-center gap-2 rounded-full px-5 py-2 text-sm font-semibold no-underline transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900';
  const variants: Record<Variant, string> = {
    primary:
      'bg-slate-800 text-white hover:bg-slate-700 dark:bg-slate-200 dark:text-slate-900 dark:hover:bg-slate-300',
    secondary:
      'bg-slate-100 text-slate-900 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-100 dark:hover:bg-slate-700',
    ghost:
      'border border-slate-400 text-slate-700 hover:bg-slate-100 dark:border-slate-600 dark:text-slate-200 dark:hover:bg-slate-800'
  };
  const cls = $derived(`${base} ${variants[variant]}`);
</script>

{#if href}
  <a {href} class={cls} {...rest as HTMLAnchorAttributes}>
    {@render children()}
  </a>
{:else}
  <button class={cls} {...rest as HTMLButtonAttributes}>
    {@render children()}
  </button>
{/if}
