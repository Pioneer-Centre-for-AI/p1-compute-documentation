<script lang="ts">
  let { class: className = '' }: { class?: string } = $props();
</script>

{#snippet icon(name: string, x: number, y: number)}
  <g transform="translate({x - 12} {y - 12})" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round">
    {#if name === 'person'}
      <circle cx="12" cy="8" r="4" />
      <path d="M4 21 Q4 14 12 14 Q20 14 20 21" />
    {:else if name === 'code'}
      <polyline points="8,6 2,12 8,18" />
      <polyline points="16,6 22,12 16,18" />
      <line x1="14" y1="4" x2="10" y2="20" />
    {:else if name === 'scales'}
      <line x1="12" y1="3" x2="12" y2="21" />
      <line x1="7" y1="21" x2="17" y2="21" />
      <line x1="4" y1="7" x2="20" y2="7" />
      <path d="M4 7 L1.5 13 H6.5 Z" />
      <path d="M20 7 L17.5 13 H22.5 Z" />
    {:else if name === 'servers'}
      <rect x="2" y="3" width="20" height="5" rx="1" />
      <rect x="2" y="10" width="20" height="5" rx="1" />
      <rect x="2" y="17" width="20" height="5" rx="1" />
    {:else if name === 'plus'}
      <circle cx="12" cy="12" r="9" stroke-dasharray="3 3" />
      <line x1="12" y1="8" x2="12" y2="16" />
      <line x1="8" y1="12" x2="16" y2="12" />
    {/if}
  </g>
{/snippet}

{#snippet tile(x: number, y: number, width: number, name: string, label: string, mono: string, dashed = false)}
  <rect {x} {y} {width} height="100" rx="10" class="tile" stroke-width="1.2" stroke-dasharray={dashed ? '4 4' : undefined} />
  {@render icon(name, x + width / 2, y + 30)}
  <text x={x + width / 2} y={y + 66} text-anchor="middle" class="label">{label}</text>
  <text x={x + width / 2} y={y + 84} text-anchor="middle" class="mono">{mono}</text>
{/snippet}

{#snippet arrow(x: number, y1: number, y2: number)}
  {@const down = y2 > y1}
  <path d="M{x} {y1} L{x} {y2}" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" fill="none" />
  <path
    d={down ? `M${x - 4.5} ${y2 - 7} L${x} ${y2} L${x + 4.5} ${y2 - 7}` : `M${x - 4.5} ${y2 + 7} L${x} ${y2} L${x + 4.5} ${y2 + 7}`}
    stroke="currentColor"
    stroke-width="1.3"
    stroke-linecap="round"
    stroke-linejoin="round"
    fill="none"
  />
{/snippet}

<div class="text-slate-700 dark:text-slate-300 {className}">
  <svg viewBox="0 0 440 520" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="h-auto w-full">
    <!-- Academia -->
    <rect x="1" y="1" width="438" height="150" rx="14" class="panel academia" stroke-width="1" />
    <text x="18" y="25" class="mono heading">ACADEMIA</text>
    {@render tile(8, 38, 136, 'person', 'Researchers', 'NEEDS AND SKILLS')}
    {@render tile(152, 38, 136, 'code', 'Research engineers', 'OPEN NETWORK')}
    {@render tile(296, 38, 136, 'scales', 'Governance board', 'P1 CONSORTIUM')}

    {@render arrow(220, 160, 202)}
    <text x="230" y="185" class="note">needs and drafts</text>

    <!-- Research community -->
    <circle cx="220" cy="260" r="52" class="hub" stroke-width="2.5" />
    <text x="220" y="248" text-anchor="middle" class="label">Research</text>
    <text x="220" y="265" text-anchor="middle" class="label">community</text>
    <text x="220" y="282" text-anchor="middle" class="mono">STANDARDS</text>

    {@render arrow(200, 320, 361)}
    <text x="190" y="344" text-anchor="end" class="note">standards</text>
    {@render arrow(240, 361, 320)}
    <text x="250" y="344" class="note">feasibility</text>

    <!-- Providers -->
    <rect x="1" y="370" width="438" height="149" rx="14" class="panel providers" stroke-width="1" />
    <text x="18" y="394" class="mono heading">PROVIDERS</text>
    {@render tile(16, 406, 200, 'servers', 'P1-funded partitions', 'DTU, NGC, GEFION')}
    <rect x="138" y="414" width="70" height="16" rx="8" class="tag" />
    <text x="173" y="425.5" text-anchor="middle" class="mono tag-text">P1 FUNDING</text>
    {@render tile(224, 406, 200, 'plus', 'Other providers', 'WELCOME TO JOIN', true)}
  </svg>
</div>

<style>
  .panel {
    fill: color-mix(in oklch, var(--color-slate-500) 5%, transparent);
    stroke: var(--color-slate-200);
  }
  .panel.academia {
    fill: color-mix(in oklch, var(--color-plum) 6%, transparent);
    stroke: color-mix(in oklch, var(--color-plum) 30%, transparent);
  }
  .tile,
  .hub {
    fill: #fff;
    stroke: var(--color-slate-300);
  }
  .hub {
    stroke: var(--color-coral);
  }
  .tag {
    fill: var(--color-highlight);
  }

  .label {
    font-size: 12.5px;
    font-weight: 600;
    fill: currentColor;
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.12em;
    fill: var(--color-slate-500);
  }
  .heading {
    font-size: 10px;
    letter-spacing: 0.16em;
  }
  .tag-text {
    font-size: 8.5px;
    fill: var(--color-slate-950);
  }
  .note {
    font-size: 11.5px;
    font-style: italic;
    fill: var(--color-slate-500);
  }

  :global(html.dark) .panel {
    stroke: var(--color-slate-700);
  }
  :global(html.dark) .panel.academia {
    fill: color-mix(in oklch, var(--color-plum-bright) 7%, transparent);
    stroke: color-mix(in oklch, var(--color-plum-bright) 30%, transparent);
  }
  :global(html.dark) .tile,
  :global(html.dark) .hub {
    fill: var(--color-slate-900);
  }
  :global(html.dark) .tile {
    stroke: var(--color-slate-700);
  }
  :global(html.dark) .mono:not(.tag-text),
  :global(html.dark) .note {
    fill: var(--color-slate-400);
  }
</style>
