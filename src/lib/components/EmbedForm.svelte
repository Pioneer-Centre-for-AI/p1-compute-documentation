<script lang="ts">
  type Props = {
    title: string;
    src: string;
    height?: string;
    minHeight?: string;
  };
  let { title, src, height = '600px', minHeight }: Props = $props();
</script>

<details class="my-4 rounded-lg border border-slate-200 dark:border-slate-700">
  <summary class="cursor-pointer rounded-lg bg-slate-50 px-4 py-3 font-medium text-slate-900 hover:bg-slate-100 dark:bg-slate-800/50 dark:text-slate-100 dark:hover:bg-slate-800">
    {title}
  </summary>
  <div class="p-3">
    <iframe
      {src}
      {title}
      class="w-full rounded border-0"
      style:height={height}
      style:min-height={minHeight ?? height}
      loading="lazy"
    ></iframe>
  </div>
</details>
