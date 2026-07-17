<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import Header from '$lib/components/Header.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';

  let { children } = $props();
  let isLanding = $derived($page.url.pathname === '/');

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
</script>

<div class="min-h-screen bg-white text-slate-700 dark:bg-slate-900 dark:text-slate-200">
  {#if isLanding}
    <div class="fixed right-4 top-4 z-30">
      <ThemeToggle />
    </div>
    {@render children()}
  {:else}
    <Header />
    <div class="mx-auto flex max-w-7xl gap-8 px-4 py-8 md:px-6">
      <aside class="hidden w-60 shrink-0 md:block">
        <div class="sticky top-20">
          <Sidebar />
        </div>
      </aside>
      <div class="min-w-0 flex-1">
        {@render children()}
      </div>
    </div>
  {/if}
</div>
