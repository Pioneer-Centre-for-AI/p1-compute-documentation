export type Os = 'linux' | 'darwin';
export type Arch = 'x64' | 'arm64';
export type Platform = { os: Os; arch: Arch };

export function currentPlatform(): Platform {
  const os: Os = process.platform === 'darwin' ? 'darwin' : 'linux';
  const arch: Arch = process.arch === 'arm64' ? 'arm64' : 'x64';
  return { os, arch };
}

export function assetName({ os, arch }: Platform): string {
  return `aicentre-${os}-${arch}.tar.gz`;
}

export function isNixStore(path: string): boolean {
  return path.startsWith('/nix/store/');
}
