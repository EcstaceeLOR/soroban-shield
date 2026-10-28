import { Link } from 'react-router-dom'
import { Page, Section } from '../components/Page'

const scanner = [
  ['CLI scanning', 'sorobanshield scan [PATH] with project discovery and validation.'],
  ['Syntax-aware analysis', 'Rust files are parsed with syn; project code is never compiled or executed.'],
  ['Text, JSON and SARIF reports', 'JSON uses a versioned schema; SARIF 2.1.0 targets code-scanning tools; output is deterministic.'],
  ['Configuration file', 'sorobanshield.toml: exclude path prefixes, disable rules, set fail_on. Unknown keys and rule IDs are errors.'],
  ['Baselines', 'scan --write-baseline / --baseline hide recorded findings so teams can fail only on new ones; suppression is always shown in the report.'],
  ['Inline suppressions', '// sorobanshield-ignore: SS-001 -- reason hides findings on a line or block; every suppression and every unused one is reported.'],
  ['Exit codes and --fail-on', 'Distinguishes findings (1), invalid usage (2) and scans that could not complete (3).'],
  ['Rule catalog commands', 'rules list and rules explain, with JSON output.'],
  ['Fixture-based tests', 'Positive, negative and edge-case fixture projects plus CLI integration tests.'],
]

const ruleSummaries = [
  ['SS-001', 'Authorization path review for public contract functions that write storage.'],
  ['SS-002', 'Arithmetic boundary review for unchecked arithmetic and narrowing casts on integer parameters.'],
  ['SS-003', 'External call review: caller-chosen call targets and discarded results of fallible external calls.'],
  ['SS-004', 'Error-path review: unwrap, expect and panic macros in public contract functions (informational).'],
  ['SS-005', 'Test suggestions: state-changing entrypoints that no test in the project mentions (informational).'],
  ['SS-006', 'Informational dependency and toolchain diagnostics read from Cargo.toml and project files.'],
]

const planned = [
  'Glob-based exclusions',
  'Multi-contract / workspace reporting',
  'Published releases and prebuilt binaries',
]

export default function Features() {
  return (
    <Page title="Features" lead="What works in this version, and what is still only a plan.">
      <Section title="Scanner">
        <ul className="grid gap-3 md:grid-cols-2">
          {scanner.map(([t, d]) => (
            <li key={t} className="card">
              <span className="font-semibold text-white">{t}</span>
              <p className="mt-1 text-sm">{d}</p>
            </li>
          ))}
        </ul>
      </Section>
      <Section title="Rules">
        <ul className="grid gap-3 md:grid-cols-2">
          {ruleSummaries.map(([id, d]) => (
            <li key={id} className="card">
              <Link to={`/rules/${id}`} className="font-mono font-semibold text-accent hover:underline">{id}</Link>
              <span className="ml-2 rounded-full border border-emerald-500/50 bg-emerald-500/10 px-2 py-0.5 text-xs text-emerald-300">implemented</span>
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
