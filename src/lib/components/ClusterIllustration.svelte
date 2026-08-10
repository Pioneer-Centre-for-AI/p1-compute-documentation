<script lang="ts">
  // Line illustrations for the three P1 clusters, adapted from the
  // compute-resources slide deck. Strokes follow currentColor so the
  // parent controls light/dark ink; coral and plum come from the theme.
  type Props = { slug: string; class?: string };
  let { slug, class: className = '' }: Props = $props();
</script>

<span class="block text-slate-700 dark:text-slate-300 {className}" aria-hidden="true">
  {#if slug === 'dtu'}
    <!-- Rack with GPU bays, laptop wired in for interactive access -->
    <svg viewBox="0 0 150 120" xmlns="http://www.w3.org/2000/svg" fill="none" class="h-auto w-full">
      <rect x="20" y="18" width="74" height="88" rx="3" stroke="currentColor" stroke-width="1.5" />
      <g stroke="currentColor" stroke-width="1.1">
        <rect x="26" y="24" width="62" height="16" rx="1" />
        <rect x="26" y="44" width="62" height="16" rx="1" />
        <rect x="26" y="64" width="62" height="16" rx="1" />
        <rect x="26" y="84" width="62" height="16" rx="1" />
      </g>
      <circle cx="31" cy="32" r="1.4" fill="var(--color-coral)" />
      <circle cx="31" cy="52" r="1.4" fill="var(--color-coral)" />
      <circle cx="31" cy="72" r="1.4" fill="var(--color-coral)" />
      <circle cx="31" cy="92" r="1.4" fill="var(--color-coral)" />
      <g stroke="var(--color-coral)" stroke-width="1">
        <rect x="52" y="28" width="14" height="8" rx="0.6" />
        <rect x="70" y="28" width="14" height="8" rx="0.6" />
        <rect x="52" y="48" width="14" height="8" rx="0.6" />
        <rect x="70" y="48" width="14" height="8" rx="0.6" />
        <rect x="52" y="68" width="14" height="8" rx="0.6" />
        <rect x="70" y="68" width="14" height="8" rx="0.6" />
        <rect x="52" y="88" width="14" height="8" rx="0.6" />
        <rect x="70" y="88" width="14" height="8" rx="0.6" />
      </g>
      <path d="M94 32 Q104 32 110 36 Q116 40 120 40" stroke="var(--color-coral)" stroke-width="1.5" stroke-linecap="round" />
      <rect x="116" y="24" width="28" height="16" rx="1.4" stroke="currentColor" stroke-width="1.4" />
      <line x1="121" y1="29" x2="139" y2="29" stroke="currentColor" stroke-width="0.7" opacity="0.45" />
      <line x1="121" y1="32" x2="135" y2="32" stroke="currentColor" stroke-width="0.7" opacity="0.45" />
      <line x1="121" y1="35" x2="137" y2="35" stroke="currentColor" stroke-width="0.7" opacity="0.45" />
      <path d="M112 40 L148 40 L146 44 L114 44 Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
    </svg>
  {:else if slug === 'ngc'}
    <!-- Compute plus large storage, marked secure with a corner lock badge -->
    <svg viewBox="0 0 150 120" xmlns="http://www.w3.org/2000/svg" fill="none" class="h-auto w-full">
      <rect x="20" y="14" width="84" height="92" rx="3" stroke="currentColor" stroke-width="1.5" />
      <g stroke="currentColor" stroke-width="1.1">
        <rect x="26" y="20" width="72" height="12" rx="1" />
        <rect x="26" y="34" width="72" height="12" rx="1" />
        <rect x="26" y="48" width="72" height="12" rx="1" />
      </g>
      <circle cx="32" cy="26" r="1.3" fill="var(--color-coral)" />
      <circle cx="32" cy="40" r="1.3" fill="var(--color-coral)" />
      <circle cx="32" cy="54" r="1.3" fill="var(--color-coral)" />
      <rect x="26" y="64" width="72" height="38" stroke="currentColor" stroke-width="1.2" />
      <g stroke="currentColor" stroke-width="0.8" opacity="0.7">
        <rect x="30" y="68" width="14" height="9" />
        <rect x="46" y="68" width="14" height="9" />
        <rect x="62" y="68" width="14" height="9" />
        <rect x="78" y="68" width="14" height="9" />
        <rect x="30" y="79" width="14" height="9" />
        <rect x="46" y="79" width="14" height="9" />
        <rect x="62" y="79" width="14" height="9" />
        <rect x="78" y="79" width="14" height="9" />
        <rect x="30" y="90" width="14" height="9" />
        <rect x="46" y="90" width="14" height="9" />
        <rect x="62" y="90" width="14" height="9" />
        <rect x="78" y="90" width="14" height="9" />
      </g>
      <g transform="translate(114 10)">
        <circle cx="14" cy="14" r="14" fill="var(--color-plum)" />
        <path d="M10 13 V10 Q10 6 14 6 Q18 6 18 10 V13" stroke="#ffffff" stroke-width="1.6" stroke-linecap="round" />
        <rect x="8" y="13" width="12" height="9" rx="1.3" fill="#ffffff" />
        <circle cx="14" cy="17" r="1.2" fill="var(--color-plum)" />
      </g>
    </svg>
  {:else if slug === 'gefion'}
    <!-- DGX-class chassis, NVLink-coupled GPUs, scale-out silhouettes, security badge -->
    <svg viewBox="0 0 150 120" xmlns="http://www.w3.org/2000/svg" fill="none" class="h-auto w-full">
      <rect x="12" y="14" width="80" height="92" rx="3" stroke="currentColor" stroke-width="1.6" />
      <line x1="12" y1="24" x2="92" y2="24" stroke="currentColor" stroke-width="1.1" />
      <circle cx="20" cy="19" r="1.5" fill="var(--color-coral)" />
      <circle cx="26" cy="19" r="1.5" fill="currentColor" opacity="0.4" />
      <circle cx="32" cy="19" r="1.5" fill="currentColor" opacity="0.4" />
      <g stroke="currentColor" stroke-width="1.2">
        <rect x="16" y="30" width="16" height="26" />
        <rect x="34" y="30" width="16" height="26" />
        <rect x="52" y="30" width="16" height="26" />
        <rect x="70" y="30" width="16" height="26" />
        <rect x="16" y="64" width="16" height="26" />
        <rect x="34" y="64" width="16" height="26" />
        <rect x="52" y="64" width="16" height="26" />
        <rect x="70" y="64" width="16" height="26" />
      </g>
      <g stroke="currentColor" stroke-width="0.7" opacity="0.4">
        <line x1="19" y1="36" x2="29" y2="36" />
        <line x1="37" y1="36" x2="47" y2="36" />
        <line x1="55" y1="36" x2="65" y2="36" />
        <line x1="73" y1="36" x2="83" y2="36" />
        <line x1="19" y1="70" x2="29" y2="70" />
        <line x1="37" y1="70" x2="47" y2="70" />
        <line x1="55" y1="70" x2="65" y2="70" />
        <line x1="73" y1="70" x2="83" y2="70" />
      </g>
      <line x1="16" y1="60" x2="86" y2="60" stroke="var(--color-coral)" stroke-width="2.5" />
      <circle cx="24" cy="60" r="2" fill="var(--color-coral)" />
      <circle cx="42" cy="60" r="2" fill="var(--color-coral)" />
      <circle cx="60" cy="60" r="2" fill="var(--color-coral)" />
      <circle cx="78" cy="60" r="2" fill="var(--color-coral)" />
      <line x1="12" y1="96" x2="92" y2="96" stroke="currentColor" stroke-width="1.1" />
      <line x1="20" y1="101" x2="32" y2="101" stroke="currentColor" stroke-width="0.9" opacity="0.5" />
      <line x1="38" y1="101" x2="50" y2="101" stroke="currentColor" stroke-width="0.9" opacity="0.5" />
      <line x1="56" y1="101" x2="68" y2="101" stroke="currentColor" stroke-width="0.9" opacity="0.5" />
      <path d="M96 60 L112 60" stroke="var(--color-coral)" stroke-width="1.6" stroke-linecap="round" />
      <path d="M109 56 L113 60 L109 64" stroke="var(--color-coral)" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      <rect x="116" y="34" width="14" height="52" rx="1.5" stroke="currentColor" stroke-width="1.1" stroke-dasharray="2.5,2" />
      <line x1="119" y1="44" x2="127" y2="44" stroke="currentColor" stroke-width="0.7" opacity="0.5" />
      <line x1="119" y1="56" x2="127" y2="56" stroke="currentColor" stroke-width="0.7" opacity="0.5" />
      <line x1="119" y1="68" x2="127" y2="68" stroke="currentColor" stroke-width="0.7" opacity="0.5" />
      <line x1="119" y1="78" x2="127" y2="78" stroke="currentColor" stroke-width="0.7" opacity="0.5" />
      <rect x="134" y="40" width="12" height="40" rx="1.5" stroke="currentColor" stroke-width="1" stroke-dasharray="2.5,2" opacity="0.6" />
      <line x1="136" y1="50" x2="144" y2="50" stroke="currentColor" stroke-width="0.6" opacity="0.35" />
      <line x1="136" y1="62" x2="144" y2="62" stroke="currentColor" stroke-width="0.6" opacity="0.35" />
      <line x1="136" y1="72" x2="144" y2="72" stroke="currentColor" stroke-width="0.6" opacity="0.35" />
      <g transform="translate(2 2)">
        <circle cx="11" cy="11" r="11" fill="var(--color-plum)" />
        <path d="M7.5 10.5 V8 Q7.5 4.5 11 4.5 Q14.5 4.5 14.5 8 V10.5" stroke="#ffffff" stroke-width="1.3" stroke-linecap="round" />
        <rect x="6" y="10.5" width="10" height="7.5" rx="1.1" fill="#ffffff" />
        <circle cx="11" cy="14" r="1" fill="var(--color-plum)" />
      </g>
    </svg>
  {/if}
</span>
