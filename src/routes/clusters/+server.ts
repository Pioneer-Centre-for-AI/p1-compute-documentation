import { movedPage } from '$lib/server/moved';
import type { RequestHandler } from './$types';

export const prerender = true;
export const trailingSlash = 'always';

export const GET: RequestHandler = () => movedPage('/providers/');
