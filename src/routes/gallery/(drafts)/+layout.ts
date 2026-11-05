import { dev } from '$app/env';
import { error } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';

export const prerender = false;

export const load: LayoutLoad = () => {
  if (!dev) error(404, 'Not found');
};
