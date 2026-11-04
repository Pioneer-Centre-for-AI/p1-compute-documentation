// /install: the aicentre installer, for `curl -fsSL https://hpc.aicentre.dk/install | sh`.
//
// The script lives with the CLI in pkgs/cli/install.sh and is served here as
// it is, so the site and the release always agree on what gets installed.
// Prerendered to a file named `install`, with no extension; the CLI's docs
// page is /cli/ for that reason.
import script from '../../../pkgs/cli/install.sh?raw';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () =>
  new Response(script, { headers: { 'content-type': 'text/plain; charset=utf-8' } });
