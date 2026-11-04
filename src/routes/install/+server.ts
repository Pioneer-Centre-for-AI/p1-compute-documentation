import script from '../../../pkgs/cli/install.sh?raw';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () =>
  new Response(script, { headers: { 'content-type': 'text/plain; charset=utf-8' } });
