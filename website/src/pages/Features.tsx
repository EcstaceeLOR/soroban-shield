import { Link } from 'react-router-dom'
import { Page, Section } from '../components/Page'

const implemented = [
  ['CLI scanning', 'sorobanshield scan [PATH] with project discovery and validation.'],
  ['Syntax-aware analysis', 'Rust files are parsed with syn; project code is never compiled or executed.'],
  ['Rule SS-001', 'Authorization path review for public contract functions that write storage.'],
  ['Baselines', 'scan --write-baseline / --baseline hide recorded findings so teams can fail only on new ones; suppression is always shown in the report.'],
  ['SARIF output', 'scan --format sarif writes SARIF 2.1.0 for code-scanning tools.'],
  ['Configuration file', 'sorobanshield.toml: exclude path prefixes, disable rules, set fail_on. Unknown keys and rule IDs are errors.'],
  ['Rule SS-003', 'External call review: caller-chosen call targets and discarded results of fallible external calls.'],
  ['Rule SS-004', 'Error-path review: unwrap, expect and panic macros in public contract functions (informational).'],
  ['Rule SS-005', 'Test suggestions: state-changing entrypoints that no test in the project mentions (informational).'],
  ['Rule SS-006', 'Informational dependency and toolchain diagnostics read from Cargo.toml and project files.'],
  ['Rule SS-002', 'Arithmetic boundary review for unchecked arithmetic and narrowing casts on integer parameters.'],
  ['Text and JSON reports', 'JSON uses a versioned schema; output is deterministic.'],
  ['Exit codes and --fail-on', 'Distinguishes findings (1), invalid usage (2) and scans that could not complete (3).'],
  ['Rule catalog commands', 'rules list and rules explain, with JSON output.'],
  ['Fixture-based tests', 'Positive, negative and edge-case fixture projects plus CLI integration tests.'],
]

const planned = [
  'Inline suppression comments and glob-based exclusions',
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
