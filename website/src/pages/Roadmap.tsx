import { Page } from '../components/Page'

type State = 'done' | 'in progress' | 'planned'
const items: { title: string; state: State; note: string }[] = [
  { title: 'CLI foundation', state: 'done', note: 'Project discovery and validation, finding model, text reporter, help and version.' },
  { title: 'First tested rule (SS-001)', state: 'done', note: 'Syntax-aware authorization path review with positive, negative and edge fixtures.' },
  { title: 'Second tested rule (SS-002)', state: 'done', note: 'Arithmetic boundary review for integer parameters of public contract functions, with positive, negative and edge fixtures.' },
  { title: 'Diagnostics rule (SS-006)', state: 'done', note: 'Informational manifest and toolchain diagnostics; no advisory database, no vulnerability claims.' },
  { title: 'JSON report and exit codes', state: 'done', note: 'Versioned JSON schema, --fail-on threshold, exit codes 0 to 3, integration tests.' },
  { title: 'Documentation website', state: 'in progress', note: 'Initial version: landing page, docs, generated rule catalog.' },
  { title: 'CI workflow', state: 'done', note: 'Repository workflow (fmt, clippy, tests, website build) ran successfully on its first push to main.' },
  { title: 'Configuration file', state: 'done', note: 'sorobanshield.toml: path-prefix exclusions, rule disabling and fail_on, strictly validated.' },
  { title: 'Baseline support', state: 'done', note: 'Record findings once and fail only on new ones; entries ignore line numbers; suppression is reported.' },
  { title: 'SARIF output', state: 'done', note: 'SARIF 2.1.0 via --format sarif; sample outputs were validated against the official schema by hand, not in CI.' },
  { title: 'Rules SS-003, SS-004 and SS-005', state: 'done', note: 'External call review, error-path review and test suggestions, each with positive, negative and edge fixtures and documented limitations.' },
  { title: 'First tagged release', state: 'planned', note: 'No release has been published. Build from source.' },
  { title: 'Independent security review', state: 'planned', note: 'Not performed.' },
]
const style: Record<State, string> = {
  done: 'border-emerald-500/50 bg-emerald-500/10 text-emerald-300',
  'in progress': 'border-amber-500/50 bg-amber-500/10 text-amber-300',
  planned: 'border-slate-500/60 bg-slate-500/10 text-slate-300',
}

export default function Roadmap() {
  return (
    <Page title="Roadmap" lead="Status of each milestone. Nothing is marked done without code and tests behind it.">
      <ul className="space-y-3">
        {items.map((i) => (
          <li key={i.title} className="card">
            <div className="flex flex-wrap items-center gap-2">
              <span className={`rounded-full border px-2.5 py-0.5 text-xs font-medium ${style[i.state]}`}>{i.state}</span>
              <h2 className="font-semibold text-white">{i.title}</h2>
            </div>
            <p className="mt-1 text-sm">{i.note}</p>
          </li>
        ))}
      </ul>
    </Page>
  )
}
