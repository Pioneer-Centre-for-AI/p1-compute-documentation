<script lang="ts">
  let { class: className = '' }: { class?: string } = $props();
</script>

{#snippet line(d: string, head: string, accent = false)}
  <path
    {d}
    stroke={accent ? 'var(--color-coral)' : 'currentColor'}
    stroke-width={accent ? 2 : 1.5}
    stroke-linecap="round"
    fill="none"
  />
  <path
    d={head}
    stroke={accent ? 'var(--color-coral)' : 'currentColor'}
    stroke-width={accent ? 2 : 1.5}
    stroke-linecap="round"
    stroke-linejoin="round"
    fill="none"
  />
{/snippet}

{#snippet head(d: string)}
  <path {d} stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" />
{/snippet}

{#snippet pair(x: number, y: number, width: number)}
  <rect {x} {y} {width} height="84" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
  <text x={x + width / 2} y={y + 26} text-anchor="middle" class="diagram-label">Research engineers</text>
  <text x={x + width / 2} y={y + 46} text-anchor="middle" class="diagram-label">and HPC governance board</text>
  <g stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" transform="translate({x + width / 2 - 56} {y + 59})">
    <line x1="9" y1="4.5" x2="3.5" y2="10" />
    <line x1="9" y1="4.5" x2="14.5" y2="10" />
    <line x1="3.5" y1="10" x2="14.5" y2="10" />
    <circle cx="9" cy="2.5" r="2.2" />
    <circle cx="2.5" cy="10.5" r="2.2" />
    <circle cx="15.5" cy="10.5" r="2.2" />
  </g>
  <text x={x + width / 2 + 12} y={y + 70} text-anchor="middle" class="diagram-mono">OPEN NETWORK</text>
{/snippet}

{#snippet rfcs(x: number, y: number, width: number)}
  <rect {x} {y} {width} height="64" rx="8" stroke="var(--color-coral)" stroke-width="2.5" fill="none" />
  <text x={x + width / 2} y={y + 31} text-anchor="middle" class="diagram-title">RFCs</text>
  <text x={x + width / 2} y={y + 51} text-anchor="middle" class="diagram-mono">STANDARDS</text>
{/snippet}

{#snippet person(x: number, y: number)}
  <g stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round">
    <circle cx={x} cy={y} r="10" />
    <path d="M{x - 20} {y + 30} Q{x - 20} {y + 12} {x} {y + 12} Q{x + 20} {y + 12} {x + 20} {y + 30}" />
  </g>
{/snippet}

{#snippet servers(x: number, y: number)}
  <g stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round">
    <rect x={x - 22} y={y} width="44" height="11" rx="2" />
    <rect x={x - 22} y={y + 15} width="44" height="11" rx="2" />
    <rect x={x - 22} y={y + 30} width="44" height="11" rx="2" />
    <line x1={x - 15} y1={y + 5.5} x2={x - 11} y2={y + 5.5} />
    <line x1={x - 15} y1={y + 20.5} x2={x - 11} y2={y + 20.5} />
    <line x1={x - 15} y1={y + 35.5} x2={x - 11} y2={y + 35.5} />
  </g>
{/snippet}

<div class="text-slate-700 dark:text-slate-300 {className}">
  <svg viewBox="0 0 760 280" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="hidden h-auto w-full sm:block">
    <!-- Researchers to engineers and board -->
    {@render person(110, 34)}
    <text x="110" y="92" text-anchor="middle" class="diagram-mono">RESEARCH COMMUNITY</text>
    {@render line('M150 48 L462 48', 'M456 43 L464 48 L456 53')}
    <text x="306" y="38" text-anchor="middle" class="diagram-note">needs, through events, networks, surveys etc.</text>
    {@render pair(470, 14, 270)}

    <!-- Engineers and board to RFCs -->
    {@render line('M605 98 L605 184', 'M600 178 L605 186 L610 178')}
    <text x="615" y="140" class="diagram-note">draft and decide</text>
    {@render rfcs(515, 192, 180)}

    <!-- RFCs to providers -->
    {@render line('M515 224 L182 224', 'M188 219 L180 224 L188 229')}
    <text x="348" y="214" text-anchor="middle" class="diagram-note">expectations</text>
    <text x="348" y="242" text-anchor="middle" class="diagram-note">compute budgets follow</text>
    {@render servers(110, 204)}
    <text x="110" y="264" text-anchor="middle" class="diagram-mono">PROVIDERS</text>

    <!-- Providers back to engineers and board -->
    <path d="M150 196 L462 88" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-dasharray="4 5" fill="none" />
    {@render head('M456 95.3 L462 88 L452.8 85.9')}
    <text x="306" y="132" text-anchor="middle" transform="rotate(-19 306 132)" class="diagram-note">what is feasible</text>

    <!-- Providers to researchers -->
    {@render line('M120 196 L120 104', 'M114 111 L120 102 L126 111', true)}
    <text x="132" y="144" class="diagram-note">compute</text>
    <text x="132" y="158" class="diagram-note">and support</text>

    <!-- Research community to providers -->
    {@render line('M100 104 L100 194', 'M94 187 L100 196 L106 187')}
    <text x="90" y="153" text-anchor="end" class="diagram-note">skilled users</text>
  </svg>

  <svg viewBox="0 0 360 448" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="h-auto w-full sm:hidden">
    <!-- Researchers to engineers and board -->
    {@render person(180, 20)}
    <text x="180" y="72" text-anchor="middle" class="diagram-mono">RESEARCH COMMUNITY</text>
    {@render line('M180 80 L180 116', 'M175 110 L180 118 L185 110')}
    <text x="190" y="96" class="diagram-note">needs: events, networks,</text>
    <text x="190" y="110" class="diagram-note">surveys, etc.</text>
    {@render pair(60, 124, 240)}

    <!-- Engineers and board to RFCs -->
    {@render line('M180 208 L180 246', 'M175 240 L180 248 L185 240')}
    <text x="190" y="232" class="diagram-note">draft and decide</text>
    {@render rfcs(110, 254, 140)}

    <!-- RFCs to providers -->
    {@render line('M180 318 L180 366', 'M175 360 L180 368 L185 360')}
    <text x="190" y="340" class="diagram-note">expectations</text>
    <text x="190" y="354" class="diagram-note">compute budgets follow</text>
    {@render servers(180, 376)}
    <text x="180" y="436" text-anchor="middle" class="diagram-mono">PROVIDERS</text>

    <!-- Providers back to engineers and board -->
    <path d="M208 396 H324 Q336 396 336 384 V178 Q336 166 324 166 H308" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-dasharray="4 5" fill="none" />
    {@render head('M314 161 L306 166 L314 171')}
    <text x="350" y="286" text-anchor="middle" transform="rotate(-90 350 286)" class="diagram-note">what is feasible</text>

    <!-- Providers to researchers -->
    {@render line('M152 396 H36 Q24 396 24 384 V44 Q24 32 36 32 H148', 'M142 26 L151 32 L142 38', true)}

    <!-- Research community to providers -->
    {@render line('M108 68 H52 Q40 68 40 80 V372 Q40 384 52 384 H152', 'M146 379 L154 384 L146 389')}
    <text x="54" y="230" text-anchor="middle" transform="rotate(-90 54 230)" class="diagram-note">skilled users</text>
    <text x="16" y="214" text-anchor="middle" transform="rotate(-90 16 214)" class="diagram-note">
      compute and support
    </text>
  </svg>
</div>

<style>
  .diagram-mono {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.14em;
    fill: var(--color-slate-500);
  }

  .diagram-title {
    font-size: 24px;
    font-weight: 600;
    fill: currentColor;
  }

  .diagram-note {
    font-size: 12px;
    font-style: italic;
    fill: var(--color-slate-500);
  }

  :global(html.dark) .diagram-mono,
  :global(html.dark) .diagram-note {
    fill: var(--color-slate-400);
  }

  .diagram-label {
    font-size: 15px;
    font-weight: 600;
    fill: currentColor;
  }
</style>
