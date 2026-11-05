import { defineParams } from '@sveltejs/kit/params';

export const params = defineParams({
  segment: (param) => (/^[a-z0-9-]+$/.test(param) ? param : undefined)
});
