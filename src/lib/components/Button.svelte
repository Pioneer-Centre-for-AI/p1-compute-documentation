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
    'inline-flex items-center justify-center gap-2 rounded-md px-4 py-2 text-sm font-semibold no-underline transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-coral)] focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900';
  const variants: Record<Variant, string> = {
    primary:
      'bg-[var(--color-coral-strong)] text-white hover:bg-[#b92238] dark:bg-[var(--color-coral)] dark:text-slate-950 dark:hover:bg-[#ff6b7a]',
    secondary:
      'bg-slate-100 text-slate-900 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-100 dark:hover:bg-slate-700',
    ghost:
      'border border-slate-300 text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800'
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
