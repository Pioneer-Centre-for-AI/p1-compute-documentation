import type { ParamMatcher } from '@sveltejs/kit';

// One plain path segment with no dot, so `/clusters/dtu.md` is left to the
// markdown route rather than caught by the old-cluster forwarding pages.
export const match: ParamMatcher = (param) => /^[a-z0-9-]+$/.test(param);
