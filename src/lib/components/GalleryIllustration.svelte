<script lang="ts">
  type Props = { slug: string; class?: string };
  let { slug, class: className = '' }: Props = $props();

  const arms = ['var(--color-coral)', 'var(--color-brand)', 'currentColor'];
  const spiral = arms.flatMap((fill, k) =>
    Array.from({ length: 14 }, (_, i) => {
      const t = i / 13;
      const r = 3 + 25 * t;
      const a = t * 2.4 * Math.PI + (k * 2 * Math.PI) / 3;
      return { x: 110 + r * Math.cos(a), y: 66 + r * Math.sin(a), fill };
    })
  );
</script>

<span class="block text-slate-700 dark:text-slate-300 {className}" aria-hidden="true">
  <svg viewBox="0 0 160 120" xmlns="http://www.w3.org/2000/svg" fill="none" class="h-auto w-full">
    {#if slug === 'int-gpu'}
      <rect x="10" y="12" width="140" height="96" rx="4" stroke="currentColor" stroke-width="1.5" />
      <line x1="10" y1="24" x2="150" y2="24" stroke="currentColor" stroke-width="1.1" />
      <g fill="currentColor" opacity="0.5">
        <circle cx="17" cy="18" r="1.4" />
        <circle cx="23" cy="18" r="1.4" />
        <circle cx="29" cy="18" r="1.4" />
      </g>
      <g stroke="currentColor" stroke-width="1.1">
        <rect x="18" y="32" width="52" height="18" rx="1.5" />
        <rect x="18" y="56" width="52" height="22" rx="1.5" />
        <rect x="18" y="84" width="52" height="16" rx="1.5" />
      </g>
      <g stroke="var(--color-brand)" stroke-width="2" stroke-linecap="round">
        <line x1="18" y1="33" x2="18" y2="49" />
        <line x1="18" y1="57" x2="18" y2="77" />
        <line x1="18" y1="85" x2="18" y2="99" />
      </g>
      <g stroke="currentColor" stroke-width="0.8" opacity="0.45" stroke-linecap="round">
        <line x1="24" y1="38" x2="60" y2="38" />
        <line x1="24" y1="43" x2="50" y2="43" />
        <line x1="24" y1="62" x2="58" y2="62" />
        <line x1="28" y1="67" x2="62" y2="67" />
        <line x1="28" y1="72" x2="46" y2="72" />
        <line x1="24" y1="90" x2="54" y2="90" />
        <line x1="24" y1="95" x2="40" y2="95" />
      </g>
      <rect x="78" y="32" width="64" height="68" rx="1.5" stroke="currentColor" stroke-width="1.1" />
      {#each spiral as p}
        <circle cx={p.x} cy={p.y} r="1.5" fill={p.fill} />
      {/each}
    {:else if slug === 'rem-lgn'}
      <rect x="6" y="26" width="64" height="42" rx="2" stroke="currentColor" stroke-width="1.4" />
      <path d="M2 68 L74 68 L71 74 L5 74 Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      <line x1="20" y1="26" x2="20" y2="68" stroke="currentColor" stroke-width="1" />
      <rect x="10" y="31" width="6" height="6" rx="1" fill="var(--color-brand)" />
      <g stroke="currentColor" stroke-width="1">
        <rect x="10" y="41" width="6" height="6" rx="1" />
        <rect x="10" y="51" width="6" height="6" rx="1" />
      </g>
      <g stroke="currentColor" stroke-width="0.8" opacity="0.45" stroke-linecap="round">
        <line x1="25" y1="33" x2="46" y2="33" />
        <line x1="29" y1="39" x2="62" y2="39" />
        <line x1="29" y1="45" x2="54" y2="45" />
        <line x1="25" y1="51" x2="42" y2="51" />
        <line x1="29" y1="57" x2="60" y2="57" />
      </g>
      <line x1="54" y1="43" x2="54" y2="47" stroke="var(--color-coral)" stroke-width="1.3" />
      <line x1="72" y1="44" x2="100" y2="44" stroke="var(--color-coral)" stroke-width="1.4" stroke-linecap="round" stroke-dasharray="3 3" />
      <rect x="82" y="38" width="9" height="8" rx="1.5" stroke="var(--color-coral)" stroke-width="1.2" />
      <path d="M84 38 L84 35.5 Q86.5 32.5 89 35.5 L89 38" stroke="var(--color-coral)" stroke-width="1.2" />
      <rect x="102" y="20" width="46" height="50" rx="3" stroke="currentColor" stroke-width="1.5" />
      <g stroke="currentColor" stroke-width="1.1">
        <rect x="108" y="27" width="34" height="11" rx="1" />
        <rect x="108" y="42" width="34" height="11" rx="1" />
      </g>
      <circle cx="113" cy="32.5" r="1.3" fill="var(--color-brand)" />
      <circle cx="113" cy="47.5" r="1.3" fill="var(--color-brand)" />
      <path d="M125 70 L125 82" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <path d="M121.5 79 L125 83 L128.5 79" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      <g stroke="currentColor" stroke-width="1.1">
        <rect x="100" y="88" width="14" height="12" rx="1.5" />
        <rect x="118" y="88" width="14" height="12" rx="1.5" />
        <rect x="136" y="88" width="14" height="12" rx="1.5" />
      </g>
      <rect x="100" y="88" width="14" height="12" rx="1.5" fill="var(--color-brand)" opacity="0.35" />
    {:else if slug === 'xfr-dat'}
      <rect x="8" y="38" width="36" height="24" rx="1.5" stroke="currentColor" stroke-width="1.4" />
      <path d="M4 62 L48 62 L46 67 L6 67 Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      <g stroke="currentColor" stroke-width="0.7" opacity="0.45" stroke-linecap="round">
        <line x1="13" y1="45" x2="38" y2="45" />
        <line x1="13" y1="50" x2="32" y2="50" />
        <line x1="13" y1="55" x2="35" y2="55" />
      </g>
      <rect x="62" y="40" width="34" height="26" rx="2" stroke="currentColor" stroke-width="1.2" stroke-dasharray="3 2.5" />
      <path d="M70 50 L75 50 L77 52 L88 52 L88 60 L70 60 Z" stroke="currentColor" stroke-width="1" stroke-linejoin="round" />
      <rect x="114" y="18" width="38" height="84" rx="3" stroke="currentColor" stroke-width="1.5" />
      <g stroke="currentColor" stroke-width="1.1">
        <rect x="120" y="26" width="26" height="14" rx="1" />
        <rect x="120" y="44" width="26" height="14" rx="1" />
        <rect x="120" y="62" width="26" height="14" rx="1" />
        <rect x="120" y="80" width="26" height="14" rx="1" />
      </g>
      <g fill="var(--color-coral)">
        <circle cx="125" cy="33" r="1.3" />
        <circle cx="125" cy="51" r="1.3" />
        <circle cx="125" cy="69" r="1.3" />
        <circle cx="125" cy="87" r="1.3" />
      </g>
      <g stroke="var(--color-coral)" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" fill="none">
        <path d="M48 32 Q56 26 64 32" />
        <path d="M61 28.5 L64 32 L59.5 33" />
        <path d="M94 32 Q104 24 112 30" />
        <path d="M109 26.5 L112 30 L107.5 31" />
      </g>
      <g stroke="var(--color-brand)" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" fill="none">
        <path d="M112 86 Q82 100 50 78" />
        <path d="M53 77 L50 78 L51.5 81.5" />
      </g>
      <rect x="78" y="88" width="8" height="10" rx="1" stroke="var(--color-brand)" stroke-width="1" />
    {:else}
      <rect x="56" y="36" width="48" height="48" rx="4" stroke="currentColor" stroke-width="1.5" />
      <rect x="67" y="47" width="26" height="26" rx="2" stroke="currentColor" stroke-width="1.1" />
      <circle cx="80" cy="60" r="3" fill="var(--color-coral)" />
      <g stroke="currentColor" stroke-width="1.1" stroke-linecap="round">
        {#each [66, 80, 94] as x}
          <line x1={x} y1="30" x2={x} y2="36" />
          <line x1={x} y1="84" x2={x} y2="90" />
        {/each}
        {#each [46, 60, 74] as y}
          <line x1="50" y1={y} x2="56" y2={y} />
          <line x1="104" y1={y} x2="110" y2={y} />
        {/each}
      </g>
    {/if}
  </svg>
</span>
