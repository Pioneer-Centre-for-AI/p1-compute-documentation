declare const AICENTRE_VERSION: string | undefined;

export const VERSION: string =
  typeof AICENTRE_VERSION === 'string' ? AICENTRE_VERSION : '0.0.0-dev';
