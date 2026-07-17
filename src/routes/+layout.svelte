<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import Header from '$lib/components/Header.svelte';
  import NavList from '$lib/components/NavList.svelte';
  import NavDrawer from '$lib/components/NavDrawer.svelte';
  import Footer from '$lib/components/Footer.svelte';

  let { children } = $props();
  let isLanding = $derived($page.url.pathname === '/');
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
</script>

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
        <div class="sticky top-20">
          <NavList />
        </div>
      </aside>
      <div class="min-w-0 flex-1">
        {@render children()}
      </div>
    </div>
  {/if}

  <Footer />
</div>
