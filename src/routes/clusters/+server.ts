// /clusters/ became /providers/. Old links land on this page and are sent on;
// see $lib/server/moved for why it is a page rather than a redirect.
import { movedPage } from '$lib/server/moved';
import type { RequestHandler } from './$types';

export const prerender = true;
// Endpoints default to 'never'. The old URL ended in a slash, so this must too.
export const trailingSlash = 'always';

export const GET: RequestHandler = () => movedPage('/providers/');
