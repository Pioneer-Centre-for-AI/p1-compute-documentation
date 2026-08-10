<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import Header from '$lib/components/Header.svelte';
  import NavList from '$lib/components/NavList.svelte';
  import NavDrawer from '$lib/components/NavDrawer.svelte';
  import PageToc from '$lib/components/PageToc.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import { markdownHref } from '$lib/markdownPages';

  let { children } = $props();
  let isLanding = $derived($page.url.pathname === '/');
  let mdHref = $derived(markdownHref($page.url.pathname));
  let mobileNavOpen = $state(false);

  onMount(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const onSystemChange = (e: MediaQueryListEvent) => {
      // Only auto-follow when the user hasn't set an explicit override.
      if (localStorage.getItem('theme')) return;
      document.documentElement.classList.toggle('dark', e.matches);
      window.dispatchEvent(new CustomEvent('themechange', { detail: { isDark: e.matches } }));
    };
    mq.addEventListener('change', onSystemChange);
    return () => mq.removeEventListener('change', onSystemChange);
  });

  onMount(() => {
    // Code block chrome is raw HTML injected with {@html}, so it cannot carry a
    // Svelte handler. One delegated listener covers every block on every page,
    // including blocks that appear after client-side navigation.
    const timers = new WeakMap<HTMLElement, ReturnType<typeof setTimeout>>();

    async function onClick(e: MouseEvent) {
      const btn = (e.target as HTMLElement | null)?.closest<HTMLElement>('[data-copy]');
      if (!btn) return;

      const code = btn.closest('.codeblock')?.querySelector('pre')?.textContent;
      if (!code) return;

      try {
        await navigator.clipboard.writeText(code);
      } catch {
        return; // Clipboard denied or unavailable; leave the button untouched.
      }

      btn.setAttribute('data-copied', '');
      btn.setAttribute('aria-label', 'Copied');
      clearTimeout(timers.get(btn));
      timers.set(
        btn,
        setTimeout(() => {
          btn.removeAttribute('data-copied');
          btn.setAttribute('aria-label', 'Copy code');
        }, 1500)
      );
    }

    document.addEventListener('click', onClick);
    return () => document.removeEventListener('click', onClick);
  });
</script>

<svelte:head>
  {#if mdHref}
    <link rel="alternate" type="text/markdown" href={mdHref} title="Markdown source" />
  {/if}
</svelte:head>

<div class="flex min-h-screen flex-col bg-white text-slate-700 dark:bg-slate-900 dark:text-slate-200">
  <Header variant={isLanding ? 'landing' : 'solid'} onMenuClick={() => (mobileNavOpen = true)} />
  <NavDrawer bind:open={mobileNavOpen} />

  {#if isLanding}
    <div class="flex-1">
      {@render children()}
    </div>
  {:else}
    <div class="mx-auto flex w-full max-w-7xl flex-1 gap-8 px-4 py-8 md:px-6">
      <aside class="hidden w-60 shrink-0 md:block">
        <div class="sticky top-24">
          <NavList />
        </div>
      </aside>
      <div class="min-w-0 flex-1">
        {@render children()}
      </div>
      <!-- Hides itself on pages with fewer than two headings, so index-style
           pages (clusters overview, forms) get no TOC. -->
      <aside class="hidden w-48 shrink-0 lg:block">
        <div class="sticky top-24">
          <PageToc {mdHref} />
        </div>
      </aside>
    </div>
  {/if}

  <Footer />
</div>
