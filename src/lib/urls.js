// Every external URL the site points at, in one place.
//
// Plain JS (not TS) because svelte.config.js is loaded directly by Node and
// imports this through the `p1:` remark plugin — same reason markdown/highlight.js
// is plain JS. `links.ts` re-exports these as named constants for Svelte code.
// Form hosting follows one rule: `survey` is the only form on the compute
// coordinator's own site, because the compute-needs consultation is a service
// that person provides while holding the role. Every other form belongs to an
// institution and is hosted accordingly, so it outlives whoever is coordinator.
export const URLS = {
  survey: 'https://survey.starostka.io/compute-needs',
  gpuAllowance: 'https://forms.cloud.microsoft/e/yN8Vxi7KTc',
  computeBudget: 'https://forms.cloud.microsoft/e/b6wcjy5kDy',
  coordinatorEmail: 'bstja@dtu.dk',
  coordinatorOrcid: 'https://orcid.org/0000-0002-2740-1651',
  governanceEmail: 'compute-governance-p1@aicentre.dk',
  profile: 'https://www.aicentre.dk/p1adb',
  github: 'https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation',
  dtuSignup: 'https://forms.cloud.microsoft/e/DG5qCfs6Wm',
  gefionSignup: 'https://forms.cloud.microsoft/e/f0gbc83GQU',
  ngcSignup: 'https://forms.cloud.microsoft/e/gtfrV7ap4C',
  // NGC issue the application form themselves; we can only point at their
  // instructions for obtaining it.
  ngcHowToApply:
    'https://www.eng.ngc.dk/research-and-international-collaboration/dngc-research-services/how-to-apply-for-access',
  accessException:
    'https://docs.google.com/forms/d/e/1FAIpQLSciaOm-CYwl48LGGZC7qlmCSPU7kfEitjGy4kvZXAXEPbo_eA/viewform',
  researchNetwork: 'https://www.deic.dk/en/danish-research-network'
};
