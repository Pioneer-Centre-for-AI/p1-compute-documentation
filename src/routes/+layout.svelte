<script lang="ts">
  import '../app.css';
  import Header from '$lib/components/Header.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';

  let { children } = $props();
</script>

<div class="min-h-screen bg-white text-slate-900 dark:bg-slate-900 dark:text-slate-100">
  <Header />
  <div class="mx-auto flex max-w-7xl gap-8 px-4 py-8 md:px-6">
    <aside class="hidden w-60 shrink-0 md:block">
      <div class="sticky top-20">
        <Sidebar />
      </div>
    </aside>
    <main class="prose min-w-0 flex-1 max-w-3xl">
      {@render children()}
    </main>
  </div>
</div>
