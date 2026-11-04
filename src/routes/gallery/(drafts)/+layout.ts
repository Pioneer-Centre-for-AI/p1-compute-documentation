// Gallery examples still in development. They render in `bun run dev` and do
// not exist in production: nothing here is prerendered, and should the static
// site's fallback page ever route to one, it answers 404.
import { dev } from '$app/environment';
import { error } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';

export const prerender = false;

export const load: LayoutLoad = () => {
  if (!dev) error(404, 'Not found');
};
