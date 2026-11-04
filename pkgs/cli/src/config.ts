import { existsSync, readFileSync } from 'node:fs';
import { CLUSTER_IDS, type ClusterId } from './check/clusters';
import type { Ctx } from './context';

export type ClusterSettings = { user?: string; alias?: string; account?: string; sync?: boolean };
export type Config = Partial<Record<ClusterId, ClusterSettings>>;

export class ConfigError extends Error {}

export function parseConfig(text: string, file: string): Config {
  let raw: unknown;
  try {
    raw = Bun.TOML.parse(text);
  } catch (error) {
    throw new ConfigError(`${file}: ${(error as Error).message}`);
  }
  const config: Config = {};
  for (const [key, value] of Object.entries(raw as Record<string, unknown>)) {
    if (!(CLUSTER_IDS as readonly string[]).includes(key)) {
      throw new ConfigError(`${file}: unknown table [${key}]; expected ${CLUSTER_IDS.join(', ')}`);
    }
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
      throw new ConfigError(`${file}: ${key} must be a table, as in [${key}]`);
    }
    const settings: ClusterSettings = {};
    for (const [field, v] of Object.entries(value as Record<string, unknown>)) {
      if (field === 'sync') {
        if (typeof v !== 'boolean')
          throw new ConfigError(`${file}: ${key}.sync must be true or false`);
        settings.sync = v;
        continue;
      }
      if (field !== 'user' && field !== 'alias' && field !== 'account') {
        throw new ConfigError(
          `${file}: unknown setting ${key}.${field}; expected user, alias, account or sync`
        );
      }
      if (typeof v !== 'string' || v === '') {
        throw new ConfigError(`${file}: ${key}.${field} must be a non-empty string`);
      }
      settings[field] = v;
    }
    config[key as ClusterId] = settings;
  }
  return config;
}

export function loadConfig(file: string): Config {
  if (!existsSync(file)) return {};
  return parseConfig(readFileSync(file, 'utf8'), file);
}

export function loadConfigOrSay(ctx: Pick<Ctx, 'paths' | 'err'>): Config | null {
  try {
    return loadConfig(ctx.paths.configFile);
  } catch (error) {
    if (!(error instanceof ConfigError)) throw error;
    ctx.err.write(`aicentre: ${error.message}\n`);
    return null;
  }
}
