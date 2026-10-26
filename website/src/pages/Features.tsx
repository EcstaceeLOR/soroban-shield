import { Link } from 'react-router-dom'
import { Page, Section } from '../components/Page'

const implemented = [
  ['CLI scanning', 'sorobanshield scan [PATH] with project discovery and validation.'],
  ['Syntax-aware analysis', 'Rust files are parsed with syn; project code is never compiled or executed.'],
  ['Rule SS-001', 'Authorization path review for public contract functions that write storage.'],
  ['Text and JSON reports', 'JSON uses a versioned schema; output is deterministic.'],
  ['Exit codes and --fail-on', 'Distinguishes findings (1), invalid usage (2) and scans that could not complete (3).'],
  ['Rule catalog commands', 'rules list and rules explain, with JSON output.'],
  ['Fixture-based tests', 'Positive, negative and edge-case fixture projects plus CLI integration tests.'],
]

const planned = [
  'Rules SS-002 to SS-006 (arithmetic, external calls, error paths, test suggestions, toolchain diagnostics)',
  'SARIF output',
  'Configuration file and rule exclusions',
  'Baseline support for introducing scanning gradually',
  'Multi-contract / workspace reporting',
  'Published releases and prebuilt binaries',
]

export default function Features() {
  return (
    <Page title="Features" lead="What works in this version, and what is still only a plan.">
      <Section title="Implemented">
        <ul className="space-y-3">
          {implemented.map(([t, d]) => (
            <li key={t} className="rounded-lg border border-ink-700 bg-ink-900 p-4">
              <span className="mr-2 rounded-full border border-emerald-500/50 bg-emerald-500/10 px-2 py-0.5 text-xs text-emerald-300">implemented</span>
              <span className="font-semibold text-white">{t}</span>
              <p className="mt-1 text-sm">{d}</p>
            </li>
          ))}
        </ul>
      </Section>
      <Section title="Planned (not available)">
        <ul className="space-y-2">
          {planned.map((p) => (
            <li key={p} className="rounded-lg border border-dashed border-ink-600 p-3 text-sm">
              <span className="mr-2 rounded-full border border-slate-500/60 bg-slate-500/10 px-2 py-0.5 text-xs text-slate-300">planned</span>
              {p}
            </li>
          ))}
        </ul>
      </Section>
      <Section title="Not goals">
        <p>SorobanShield does not guarantee security, replace audits, auto-fix contracts or upload source code. See <Link className="text-accent underline" to="/docs/limitations">limitations</Link>.</p>
      </Section>
    </Page>
  )
}
