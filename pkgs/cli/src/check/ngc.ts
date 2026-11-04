import type { Group } from './types';
import type { CheckEnv } from './env';

export async function checkNgc(g: Group, _c: CheckEnv): Promise<void> {
  await g.step('access', 'Remote desktop', () => ({
    status: 'info',
    detail: 'you work on NGC through its Omnissa remote desktop, which this check cannot see'
  }));
}
