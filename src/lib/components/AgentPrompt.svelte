<script lang="ts">
  import { INSTALL_PROMPT } from '#lib/installPrompt.ts';
  import { COPY_BUTTON } from '#lib/markdown/copyButton.js';
</script>

<div class="codeblock group not-prose relative my-3 overflow-hidden rounded-lg border border-slate-200 text-sm dark:border-slate-700">
  <div
    class="flex h-8 items-center gap-2 border-b border-slate-200 bg-slate-100 px-3 font-mono text-xs text-slate-700 dark:border-slate-700 dark:bg-slate-800/70 dark:text-slate-300"
  >
    <span class="opacity-60">prompt</span>
  </div>
  {@html COPY_BUTTON}
  <pre class="m-0 whitespace-pre-wrap break-words px-4 py-3 font-mono text-slate-800 dark:text-slate-100">{INSTALL_PROMPT}</pre>
</div>
