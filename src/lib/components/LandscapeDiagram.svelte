<script lang="ts">
  // The compute-landscape fan-out from the P1 slide deck: a researcher
  // enters through P1, which consults, guides, and points to the three
  // tiers of the wider HPC landscape.
  //
  // `wide` is the original slide composition (760x210), `stacked` is a
  // squarer arrangement for narrow columns like the landing hero.
  type Props = { variant?: 'wide' | 'stacked'; class?: string };
  let { variant = 'wide', class: className = '' }: Props = $props();
</script>

<div class="text-slate-700 dark:text-slate-300 {className}">
  {#if variant === 'wide'}
    <svg viewBox="0 0 760 210" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="h-auto w-full">
      <!-- Researcher -->
      <g stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round">
        <circle cx="46" cy="80" r="10" />
        <path d="M26 110 Q26 92 46 92 Q66 92 66 110" />
      </g>
      <text x="46" y="130" text-anchor="middle" class="diagram-mono">RESEARCHER</text>

      <!-- Arrow: researcher into P1 -->
      <path d="M74 96 L138 96" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" fill="none" />
      <path d="M132 91 L140 96 L132 101" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" />

      <!-- P1 entry point -->
      <rect x="146" y="64" width="140" height="68" rx="8" stroke="var(--color-coral)" stroke-width="2.5" fill="none" />
      <text x="216" y="95" text-anchor="middle" class="diagram-title">P1</text>
      <text x="216" y="114" text-anchor="middle" class="diagram-mono">ENTRY POINT</text>
      <text x="216" y="156" text-anchor="middle" class="diagram-note">consult and guide</text>

      <!-- Fan-out to the three tiers -->
      <path d="M286 88 C 360 88, 400 28, 470 28" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" fill="none" />
      <path d="M286 98 L470 98" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" fill="none" />
      <path d="M286 108 C 360 108, 400 168, 470 168" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" fill="none" />
      <path d="M462 22 L474 28 L462 34" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
      <path d="M462 92 L474 98 L462 104" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
      <path d="M462 162 L474 168 L462 174" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
      <circle cx="286" cy="98" r="3" fill="var(--color-coral)" />

      <!-- University HPCs -->
      <g transform="translate(484 6)">
        <rect x="0" y="0" width="270" height="44" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
        <g transform="translate(14 10)" stroke="currentColor" stroke-width="1.3" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="2,9 14,2 26,9" />
          <line x1="2" y1="25" x2="26" y2="25" />
          <line x1="6" y1="10" x2="6" y2="23" />
          <line x1="14" y1="10" x2="14" y2="23" />
          <line x1="22" y1="10" x2="22" y2="23" />
        </g>
        <text x="56" y="28" class="diagram-label">University HPCs</text>
      </g>

      <!-- National HPCs -->
      <g transform="translate(484 76)">
        <rect x="0" y="0" width="270" height="44" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
        <g transform="translate(14 10)" stroke="currentColor" stroke-width="1.3" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <line x1="4" y1="0" x2="4" y2="25" />
          <path d="M4 3 L24 3 L20 10 L24 17 L4 17" />
        </g>
        <text x="56" y="28" class="diagram-label">National HPCs</text>
      </g>

      <!-- International HPCs -->
      <g transform="translate(484 146)">
        <rect x="0" y="0" width="270" height="44" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
        <g transform="translate(14 10)" stroke="currentColor" stroke-width="1.3" fill="none">
          <circle cx="13" cy="13" r="11" />
          <ellipse cx="13" cy="13" rx="4.5" ry="11" />
          <line x1="2" y1="13" x2="24" y2="13" />
        </g>
        <text x="56" y="28" class="diagram-label">International HPCs</text>
      </g>
    </svg>
  {:else}
    <svg viewBox="0 0 480 384" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="h-auto w-full">
      <!-- Researcher -->
      <g stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round">
        <circle cx="46" cy="78" r="10" />
        <path d="M26 108 Q26 90 46 90 Q66 90 66 108" />
      </g>
      <text x="46" y="128" text-anchor="middle" class="diagram-mono">RESEARCHER</text>

      <!-- Arrow: researcher into P1 -->
      <path d="M74 94 L134 94" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" fill="none" />
      <path d="M128 89 L136 94 L128 99" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" />

      <!-- P1 entry point -->
      <rect x="142" y="62" width="140" height="64" rx="8" stroke="var(--color-coral)" stroke-width="2.5" fill="none" />
      <text x="212" y="92" text-anchor="middle" class="diagram-title">P1</text>
      <text x="212" y="111" text-anchor="middle" class="diagram-mono">ENTRY POINT</text>
      <text x="320" y="152" text-anchor="middle" class="diagram-note">consult and guide</text>

      <!-- Trunk from P1 down, branching into the three tiers -->
      <circle cx="212" cy="130" r="3" fill="var(--color-coral)" />
      <path d="M212 130 V190 Q212 202 224 202" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" fill="none" />
      <path d="M212 130 V260 Q212 272 224 272" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" fill="none" />
      <path d="M212 130 V330 Q212 342 224 342" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" fill="none" />
      <path d="M218 196 L227 202 L218 208" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
      <path d="M218 266 L227 272 L218 278" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
      <path d="M218 336 L227 342 L218 348" stroke="var(--color-coral)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none" />

      <!-- University HPCs -->
      <g transform="translate(234 180)">
        <rect x="0" y="0" width="230" height="44" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
        <g transform="translate(14 10)" stroke="currentColor" stroke-width="1.3" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="2,9 14,2 26,9" />
          <line x1="2" y1="25" x2="26" y2="25" />
          <line x1="6" y1="10" x2="6" y2="23" />
          <line x1="14" y1="10" x2="14" y2="23" />
          <line x1="22" y1="10" x2="22" y2="23" />
        </g>
        <text x="56" y="28" class="diagram-label">University HPCs</text>
      </g>

      <!-- National HPCs -->
      <g transform="translate(234 250)">
        <rect x="0" y="0" width="230" height="44" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
        <g transform="translate(14 10)" stroke="currentColor" stroke-width="1.3" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <line x1="4" y1="0" x2="4" y2="25" />
          <path d="M4 3 L24 3 L20 10 L24 17 L4 17" />
        </g>
        <text x="56" y="28" class="diagram-label">National HPCs</text>
      </g>

      <!-- International HPCs -->
      <g transform="translate(234 320)">
        <rect x="0" y="0" width="230" height="44" rx="6" stroke="currentColor" stroke-width="1.5" fill="none" />
        <g transform="translate(14 10)" stroke="currentColor" stroke-width="1.3" fill="none">
          <circle cx="13" cy="13" r="11" />
          <ellipse cx="13" cy="13" rx="4.5" ry="11" />
          <line x1="2" y1="13" x2="24" y2="13" />
        </g>
        <text x="56" y="28" class="diagram-label">International HPCs</text>
      </g>
    </svg>
  {/if}
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

  /* slate-500 is the muted tone on white, but only reaches ~3.4:1 on the
     slate-900 page background, and this text is 10-12px. */
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
