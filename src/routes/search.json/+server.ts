import { searchIndex } from '#lib/markdown/search.ts';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () =>
  new Response(JSON.stringify(searchIndex()), {
    headers: { 'content-type': 'application/json; charset=utf-8' }
  });
