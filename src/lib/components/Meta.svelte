<script lang="ts">
  // One place for every per-page head tag, so a new page cannot ship with a
  // title but no description, or a description that never reaches Open Graph.
  //
  // Pages previously wrote <title> and the description by hand and had neither
  // og: nor twitter: tags at all, which meant a link pasted into Slack (the
  // channel this site points researchers at) unfurled as a bare URL.
  import { page } from '$app/stores';
  import { SITE_NAME, SITE_URL } from '$lib/site';

  type Props = {
    title: string;
    description: string;
    /** 'article' for a page about one thing, 'website' for an index. */
    type?: 'website' | 'article';
    /** Absolute or root-relative image for link previews. */
    image?: string;
    /** Describes the image for people using a screen reader on the unfurled link. */
    imageAlt?: string;
    /** Set on the landing page, whose title is already the site name. */
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

  // Describes the default card. A page passing its own `image` should pass
  // `imageAlt` too; falling back to this would then describe the wrong picture.
  const DEFAULT_IMAGE_ALT =
    'Pioneer Centre for AI · Compute. Dedicated HPC partitions at DTU, NGC, and Gefion.';

  // "Privacy · Pioneer Centre for AI · Compute" reads long in a tab, so the
  // suffix is the short form. The landing page is already the full name.
  const fullTitle = $derived(isLanding ? title : `${title} · P1 Compute`);
  const canonical = $derived(`${SITE_URL}${$page.url.pathname}`);
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
