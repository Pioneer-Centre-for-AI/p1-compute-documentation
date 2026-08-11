// Site-level identity, shared by the endpoints that need to emit absolute URLs
// or describe the site to a machine: sitemap.xml, llms.txt, llms-full.txt.
export const SITE_URL = 'https://hpc.aicentre.dk';

export const SITE_NAME = 'Pioneer Centre for AI · Compute';

// The one-line summary an LLM reads first. Keep it in the site's own framing:
// P1 funds and helps run partitions, it does not own the facilities.
export const SITE_SUMMARY =
  'Documentation for the compute the Pioneer Centre for AI (P1) provides to its affiliates. ' +
  'P1 funds and helps run dedicated partitions at three Danish HPC facilities (DTU, NGC, Gefion), ' +
  'with a compute coordinator working alongside each provider team. This site covers how to get ' +
  'access, choose a cluster, and get help; the deep technical ' +
  'documentation belongs to the providers and is linked from each cluster page.';
