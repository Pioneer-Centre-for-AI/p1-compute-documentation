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
  profile: 'https://www.aicentre.dk/p1adb',
  github: 'https://github.com/Pioneer-Centre-for-AI/p1-compute-documentation',
  dtuSignup:
    'https://forms.office.com/pages/responsepage.aspx?id=I_FR8s7JjkSSdzS7KFkR2TcQSfcccxdDtaq8HEbAO1RUQjZUUEoxSlRCRk5KV09WSlRZNjNLVE1KRi4u',
  gefionSignup:
    'https://forms.office.com/Pages/ResponsePage.aspx?id=I_FR8s7JjkSSdzS7KFkR2cHxzPsQ0cdIuQChdDR2mhJUNVdUNU9KUU9SSFZXNjVDWExTVEFPMElWTi4u',
  ngcUserForm:
    'https://www.ngc.dk/Media/638139612542456428/Oprettelse%20af%20Ekstern%20Bruger%20p%C3%A5%20NGC%20Infrastruktur%200.4.pdf',
  accessException:
    'https://docs.google.com/forms/d/e/1FAIpQLSciaOm-CYwl48LGGZC7qlmCSPU7kfEitjGy4kvZXAXEPbo_eA/viewform',
  researchNetwork: 'https://www.deic.dk/en/danish-research-network'
};
