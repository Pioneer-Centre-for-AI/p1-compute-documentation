<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import Header from '#lib/components/Header.svelte';
  import NavList from '#lib/components/NavList.svelte';
  import NavDrawer from '#lib/components/NavDrawer.svelte';
  import PageToc from '#lib/components/PageToc.svelte';
  import Footer from '#lib/components/Footer.svelte';
  import { markdownHref } from '#lib/markdownPages.ts';
  import { GALLERY_HREF } from '#lib/gallery.ts';

  let { children } = $props();
  let isLanding = $derived(page.url.pathname === '/');
  let isWide = $derived(isLanding || page.url.pathname === GALLERY_HREF);
  let mdHref = $derived(markdownHref(page.url.pathname));
  let mobileNavOpen = $state(false);

  onMount(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const onSystemChange = (e: MediaQueryListEvent) => {
      if (localStorage.getItem('theme')) return;
      document.documentElement.classList.toggle('dark', e.matches);
      window.dispatchEvent(new CustomEvent('themechange', { detail: { isDark: e.matches } }));
    };
    mq.addEventListener('change', onSystemChange);
    return () => mq.removeEventListener('change', onSystemChange);
  });

  onMount(() => {
    const timers = new WeakMap<HTMLElement, ReturnType<typeof setTimeout>>();

    async function onClick(e: MouseEvent) {
      const btn = (e.target as HTMLElement | null)?.closest<HTMLElement>('[data-copy]');
      if (!btn) return;

      const code = btn.closest('.codeblock')?.querySelector('pre')?.textContent;
      if (!code) return;

      try {
        await navigator.clipboard.writeText(code);
      } catch {
        return;
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
  <a
    href="#main"
    class="sr-only rounded-md bg-white px-3 py-2 text-sm font-medium text-slate-900 shadow-lg focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-50 dark:bg-slate-800 dark:text-slate-100"
  >
    Skip to content
  </a>
  <Header variant={isLanding ? 'landing' : 'solid'} onMenuClick={() => (mobileNavOpen = true)} />
  <NavDrawer bind:open={mobileNavOpen} />

  {#if isWide}
    <main id="main" class="flex-1">
      {@render children()}
    </main>
  {:else}
    <div class="mx-auto flex w-full max-w-7xl flex-1 gap-8 px-4 py-8 md:px-6">
      <aside class="hidden w-60 shrink-0 md:block">
        <div class="sticky top-24">
          <NavList />
        </div>
      </aside>
      <main id="main" class="min-w-0 flex-1">
        {@render children()}
      </main>
      <aside class="hidden w-48 shrink-0 lg:block">
        <div class="sticky top-24">
          <PageToc {mdHref} />
        </div>
      </aside>
    </div>
  {/if}

  <Footer />
</div>
