<script lang="ts">
  // Forskningsnettet drawn as what it physically is: one high-bandwidth wire
  // with facilities tied into it. The facilities hosting a P1 partition sit
  // above the wire in their brand colours, the other documented ones below.
  //
  // Membership is deliberately limited to systems this site already documents.
  // To add or remove a facility, edit these two lists and nothing else.
  type Node = { label: string; x: number; color?: string };

  const above: Node[] = [
    { label: 'P1 DTU HPC', x: 150, color: 'var(--color-brand-dtu)' },
    { label: 'P1 NGC HPC', x: 380, color: 'var(--color-brand-ngc)' },
    { label: 'P1 Gefion HPC', x: 610, color: 'var(--color-brand-gefion)' }
  ];

  const below: Node[] = [
    { label: 'UCPH', x: 110 },
    { label: 'SDU', x: 300 },
    { label: 'Aarhus', x: 490 },
    { label: 'Computerome', x: 680 }
  ];

  type Props = { class?: string };
  let { class: className = '' }: Props = $props();

  const ABOVE_W = 150;
  const BELOW_W = 130;
</script>

<div class="text-slate-700 dark:text-slate-300 {className}">
  <svg
    viewBox="0 0 760 236"
    xmlns="http://www.w3.org/2000/svg"
    role="img"
    aria-label="Forskningsnettet, the Danish research network, drawn as a single high-bandwidth link with the P1 DTU HPC, NGC HPC and Gefion HPC clusters connected above it and the UCPH, SDU, Aarhus and Computerome facilities below it."
    class="h-auto w-full"
  >
    <text x="40" y="16" class="wire-label">FORSKNINGSNETTET · DeiC</text>

    <!-- Facilities above the wire: those hosting a P1 partition -->
    {#each above as node}
      <g>
        <rect
          x={node.x - ABOVE_W / 2}
          y="32"
          width={ABOVE_W}
          height="44"
          rx="6"
          stroke={node.color ?? 'currentColor'}
          stroke-width="2"
          fill="none"
        />
        <text x={node.x} y="60" text-anchor="middle" class="node-label">{node.label}</text>
        <path
          d="M{node.x} 76 V112"
          stroke={node.color ?? 'currentColor'}
          stroke-width="1.5"
          stroke-linecap="round"
          fill="none"
        />
        <circle cx={node.x} cy="112" r="3.5" fill={node.color ?? 'currentColor'} />
      </g>
    {/each}

    <!-- The wire itself -->
    <rect x="40" y="112" width="680" height="12" rx="6" fill="var(--color-coral)" />

    <!-- Facilities below the wire -->
    {#each below as node}
      <g>
        <circle cx={node.x} cy="124" r="3.5" fill="var(--color-coral)" />
        <path
          d="M{node.x} 124 V160"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          fill="none"
        />
        <rect
          x={node.x - BELOW_W / 2}
          y="160"
          width={BELOW_W}
          height="44"
          rx="6"
          stroke="currentColor"
          stroke-width="1.5"
          fill="none"
        />
        <text x={node.x} y="188" text-anchor="middle" class="node-label">{node.label}</text>
      </g>
    {/each}

    <text x="380" y="228" text-anchor="middle" class="wire-note">
      shared high-bandwidth links between Danish research institutions
    </text>
  </svg>
</div>

<style>
  .wire-label {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.14em;
    fill: var(--color-slate-500);
  }

  .wire-note {
    font-size: 12px;
    font-style: italic;
    fill: var(--color-slate-500);
  }

  .node-label {
    font-size: 14px;
    font-weight: 600;
    fill: currentColor;
  }

  /* Same contrast lift as LandscapeDiagram: slate-500 is too dim on slate-900. */
  :global(html.dark) .wire-label,
  :global(html.dark) .wire-note {
    fill: var(--color-slate-400);
  }
</style>
