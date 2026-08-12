// External URLs used across the site. The values live in ./urls.js, which is
// plain JS so svelte.config.js can import it for the `p1:` markdown scheme.
// These named re-exports are what Svelte components import.
import { URLS } from './urls.js';

export const SURVEY_URL = URLS.survey;
export const SURVEY_PAGE = URLS.surveyPage;
export const GPU_ALLOWANCE_URL = URLS.gpuAllowance;
export const COMPUTE_BUDGET_URL = URLS.computeBudget;
export const BOOKING_URL = URLS.booking;
export const PROFILE_URL = URLS.profile;
export const GITHUB_URL = URLS.github;
export const COORDINATOR_EMAIL = URLS.coordinatorEmail;
export const COORDINATOR_ORCID = URLS.coordinatorOrcid;
export const GOVERNANCE_EMAIL = URLS.governanceEmail;

export const DTU_SIGNUP_FORM_URL = URLS.dtuSignup;
export const ACCESS_EXCEPTION_FORM_URL = URLS.accessException;
export const GEFION_SIGNUP_FORM_URL = URLS.gefionSignup;
export const NGC_SIGNUP_FORM_URL = URLS.ngcSignup;
export const NGC_HOW_TO_APPLY_URL = URLS.ngcHowToApply;

export type FormInfo = {
  title: string;
  purpose: string;
  provider: string;
  url: string;
};

// Every form, survey and booking referenced on the site, in the order a new
// researcher typically meets them. Rendered as a table on /forms/.
//
// The booking page is not a form, which is why the page is called Forms &
// Bookings rather than Forms & Surveys: that page is where a researcher looks
// for "what do I have to fill in", and leaving the consultation out of it hid
// the entry point that needs the least filling in of all. Renaming the label
// was the cheaper fix than keeping the table honest by omission.
export const FORMS: FormInfo[] = [
  {
    title: 'Book a consultation',
    purpose:
      'Meet the compute coordinator in person or online. Bring a workload, a cluster choice, a slow job, or a budget you have not committed yet.',
    provider: 'book.starostka.io',
    url: URLS.booking
  },
  {
    title: 'Your Compute Needs survey',
    purpose: 'Tell the compute coordinator about your workload, needs, and expectations. Optional, and the easiest way to get help choosing.',
    provider: 'Microsoft Forms',
    url: SURVEY_URL
  },
  {
    title: 'P1 profile and affiliation',
    purpose: 'Register or update your P1 profile and request affiliate membership. Required before any cluster access.',
    provider: 'aicentre.dk',
    url: PROFILE_URL
  },
  {
    title: 'DTU account signup',
    purpose: 'Request a user account on the P1 DTU HPC cluster.',
    provider: 'Microsoft Forms',
    url: DTU_SIGNUP_FORM_URL
  },
  {
    title: 'Gefion sandbox signup',
    purpose: 'Request access to the P1 Gefion sandbox.',
    provider: 'Microsoft Forms',
    url: GEFION_SIGNUP_FORM_URL
  },
  {
    title: 'NGC application form',
    purpose:
      'How to obtain the NGC user application form. NGC issue it themselves; sign it and email it to the compute coordinator.',
    provider: 'NGC',
    url: NGC_HOW_TO_APPLY_URL
  },
  {
    title: 'NGC access request',
    purpose: 'Tell the compute coordinator you are ready for an NGC account, once the paperwork is in place.',
    provider: 'Microsoft Forms',
    url: NGC_SIGNUP_FORM_URL
  },
  {
    title: 'Access exception request',
    purpose: 'For MSc students, research assistants, and similar roles who need cluster access with a written approval from a responsible P1 co-lead or faculty member.',
    provider: 'Google Forms',
    url: ACCESS_EXCEPTION_FORM_URL
  },
  {
    title: 'Temporary GPU allowance increase',
    purpose: 'Request a temporary raise of your GPU limit while a cluster is underutilized.',
    provider: 'Microsoft Forms',
    url: GPU_ALLOWANCE_URL
  },
  {
    title: 'Investing your compute budget',
    purpose: 'Planning to spend a grant on compute? Get advice on the split between cluster hardware, storage, cloud, and workstations before committing the money.',
    provider: 'Microsoft Forms',
    url: COMPUTE_BUDGET_URL
  }
];
