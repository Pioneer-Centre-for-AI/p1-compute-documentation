export const CLUSTER_IDS = ['dtu', 'gefion', 'ngc'] as const;
export type ClusterId = (typeof CLUSTER_IDS)[number];

export const SITE = 'https://hpc.aicentre.dk';

export const DTU = {
  name: 'P1 DTU HPC',
  alias: 'dtu',
  login: 'login9.hpc.dtu.dk',
  port: 22,
  vpn: 'vpn.dtu.dk',
  projectDir: '/dtu/p1',
  docs: `${SITE}/providers/dtu/`
} as const;

export const GEFION = {
  name: 'P1 Gefion HPC',
  alias: 'gefion',
  login: 'slurm-login-01.hpc.ite.dcai.dk',
  port: 3022,
  proxy: 'secureremote.dcai.dk',
  proxyPort: 443,
  account: 'iu_0096',
  docs: `${SITE}/providers/gefion/`
} as const;

export const NGC = {
  name: 'P1 NGC HPC',
  docs: `${SITE}/providers/ngc/`
} as const;

export const CLUSTER_NAMES: Record<ClusterId, string> = {
  dtu: DTU.name,
  gefion: GEFION.name,
  ngc: NGC.name
};

export const SHORT_NAMES: Record<ClusterId, string> = { dtu: 'DTU', gefion: 'Gefion', ngc: 'NGC' };

export function teleportKey(tsh: string, user: string): string {
  return `${tsh}/keys/${GEFION.proxy}/${user}`;
}

export function teleportCert(tsh: string, user: string): string {
  return `${tsh}/keys/${GEFION.proxy}/${user}-ssh/${GEFION.proxy}-cert.pub`;
}

export const DTU_VPN_HINT = `From outside DTU's network, connect to the DTU VPN (${DTU.vpn}) first.`;

export function tshLogin(user?: string): string {
  return `tsh login --proxy ${GEFION.proxy}${user ? ` --user ${user}` : ''}`;
}

export function hostBlock(cluster: 'dtu' | 'gefion', alias: string, user?: string): string[] {
  if (cluster === 'dtu') {
    return [
      `Host ${alias}`,
      `  HostName ${DTU.login}`,
      `  User ${user ?? '<dtu-username>'}`,
      `  ServerAliveInterval 60`
    ];
  }
  const u = user ?? '<gefion-username>';
  return [
    `Host ${alias}`,
    `  Port ${GEFION.port}`,
    `  User ${u}`,
    `  HostName ${GEFION.login}`,
    `  ProxyCommand tsh proxy ssh --cluster=${GEFION.proxy} --proxy=${GEFION.proxy}:${GEFION.proxyPort} %r@%h:%p`,
    `  CertificateFile ~/.tsh/keys/${GEFION.proxy}/${u}-ssh/${GEFION.proxy}-cert.pub`,
    `  ConnectTimeout 30`,
    `  ControlMaster no`,
    `  ControlPersist no`,
    `  IdentityFile ~/.tsh/keys/${GEFION.proxy}/${u}`,
    `  UserKnownHostsFile ~/.tsh/known_hosts`
  ];
}
