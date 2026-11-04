// Every external URL the site points at, in one place.
//
// Plain JS (not TS) because svelte.config.js is loaded directly by Node and
// imports this through the `p1:` remark plugin — same reason markdown/highlight.js
// is plain JS. `links.ts` re-exports these as named constants for Svelte code.
// Every form but the access exception request is institution-hosted
// (Microsoft Forms), so each one outlives whoever holds the coordinator role.
// The access exception request is on Google Forms. `booking` is a deliberate
// exception too: a calendar belongs to a person, not to an institution, so it
// cannot be role-hosted. Whoever holds the role next replaces the URL.
export const URLS = {
  // The raw form. Only the /survey/ page and the /forms/ directory link here
  // directly; every call to action on the site points at `surveyPage` instead,
  // so the form opens in the site's own chrome rather than a new tab.
  survey: 'https://forms.cloud.microsoft/e/Np1yaVKBup',
  surveyPage: '/survey/',
  gpuAllowance: 'https://forms.cloud.microsoft/e/yN8Vxi7KTc',
  computeBudget: 'https://forms.cloud.microsoft/e/b6wcjy5kDy',
  // Cal.com booking page for a 15 minute consultation with the coordinator,
  // in person or online. Paired with the compute-needs survey by design: the
  // survey and the slug carry the same name.
  booking: 'https://book.starostka.io/benjamin/compute-needs',
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
  // Google Forms. The short link is the one to share and open; the embed
  // takes the full form URL it redirects to, with Google's embed parameter.
  accessException: 'https://forms.gle/jSp12oBU2BcM6GJR7',
  accessExceptionEmbed:
    'https://docs.google.com/forms/d/e/1FAIpQLSciaOm-CYwl48LGGZC7qlmCSPU7kfEitjGy4kvZXAXEPbo_eA/viewform?embedded=true',
  researchNetwork: 'https://www.deic.dk/en/danish-research-network'
};
