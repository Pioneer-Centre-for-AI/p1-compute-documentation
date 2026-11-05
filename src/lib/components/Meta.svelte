<script lang="ts">
  import { page } from '$app/state';
  import { SITE_NAME, SITE_URL } from '#lib/site.ts';

  type Props = {
    title: string;
    description: string;
    type?: 'website' | 'article';
    image?: string;
    imageAlt?: string;
    isLanding?: boolean;
  };
  let {
    title,
    description,
    type = 'website',
    image,
    imageAlt,
    isLanding = false
  }: Props = $props();

  const DEFAULT_IMAGE_ALT =
    'Pioneer Centre for AI · Compute. Dedicated HPC partitions at DTU, NGC, and Gefion.';

  const fullTitle = $derived(isLanding ? title : `${title} · P1 Compute`);
  const canonical = $derived(`${SITE_URL}${page.url.pathname}`);
  const imageUrl = $derived(
    image ? (image.startsWith('http') ? image : `${SITE_URL}${image}`) : `${SITE_URL}/assets/og.png`
  );
  const alt = $derived(imageAlt ?? (image ? description : DEFAULT_IMAGE_ALT));
</script>

<svelte:head>
  <title>{fullTitle}</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={canonical} />

  <meta property="og:type" content={type} />
  <meta property="og:site_name" content={SITE_NAME} />
  <meta property="og:locale" content="en" />
  <meta property="og:title" content={fullTitle} />
  <meta property="og:description" content={description} />
  <meta property="og:url" content={canonical} />
  <meta property="og:image" content={imageUrl} />
  <meta property="og:image:alt" content={alt} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />

  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={fullTitle} />
  <meta name="twitter:description" content={description} />
  <meta name="twitter:image" content={imageUrl} />
  <meta name="twitter:image:alt" content={alt} />
</svelte:head>
