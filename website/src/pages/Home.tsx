import { Link } from 'react-router-dom'
import { CodeBlock } from '../components/CodeBlock'
import { ConfidenceBadge, SeverityBadge, StatusBadge } from '../components/Badges'
import { implementedRules, rules } from '../lib/rules'
import { REPO_URL } from '../lib/site'
import sample from '../data/sample-scan.generated.txt?raw'

const steps = [
  ['Discover', 'Find a Cargo project that declares soroban-sdk and collect its Rust files.'],
  ['Analyze', 'Parse each file and run the rules that are implemented in this release.'],
  ['Review', 'Read findings with severity, confidence, location, next steps and limitations.'],
  ['Verify', 'Confirm with tests, code review and security expertise. A scan is one input.'],
]

export default function Home() {
  const implemented = implementedRules()
  const plannedCount = rules.length - implemented.length
  return (
    <>
      <section className="border-b border-ink-700 bg-gradient-to-b from-ink-900 to-ink-950">
        <div className="mx-auto grid max-w-6xl items-center gap-10 px-4 py-16 lg:grid-cols-2">
          <div>
            <p className="font-mono text-sm text-accent">Early-stage · open source · runs locally</p>
            <h1 className="mt-3 text-4xl font-bold leading-tight text-white md:text-5xl">
              Find the risks. Understand the code. Strengthen your contracts.
            </h1>
            <p className="mt-5 text-lg text-slate-300">
              SorobanShield is intended to help Soroban developers identify code patterns that deserve closer review. It does not prove a contract is secure.
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Link to="/docs/quickstart" className="rounded-md bg-accent px-5 py-2.5 font-semibold text-ink-950 hover:bg-accent-dim">
                Get Started
              </Link>
              <Link to="/rules" className="rounded-md border border-ink-600 px-5 py-2.5 font-semibold text-white hover:bg-ink-800">
                Explore Security Rules
              </Link>
            </div>
          </div>
          <div>
            <CodeBlock
              label="$ sorobanshield scan tests/fixtures/positive/unprotected_write   (exit code 1)"
              code={sample.trimEnd()}
              wrap
            />
            <p className="text-xs text-slate-400">
              Real output of the CLI run against a fixture project in the repository, not a sample from a user project.
            </p>
          </div>
        </div>
      </section>

      <div className="mx-auto max-w-6xl space-y-16 px-4 py-14">
        <section aria-labelledby="how">
          <h2 id="how" className="text-2xl font-bold text-white">How it works</h2>
          <ol className="mt-6 grid gap-4 md:grid-cols-4">
            {steps.map(([t, d], i) => (
              <li key={t} className="rounded-lg border border-ink-700 bg-ink-900 p-4">
                <span className="font-mono text-sm text-accent">0{i + 1}</span>
                <h3 className="mt-1 font-semibold text-white">{t}</h3>
                <p className="mt-1 text-sm text-slate-300">{d}</p>
              </li>
            ))}
          </ol>
          <p className="mt-4 text-sm"><Link className="text-accent underline" to="/how-it-works">Read the full workflow</Link></p>
        </section>

        <section aria-labelledby="features">
          <h2 id="features" className="text-2xl font-bold text-white">What exists today</h2>
          <ul className="mt-6 grid gap-4 md:grid-cols-3">
            {[
              ['Local CLI', 'Scans a project directory from the terminal. No network access, no telemetry, no project code is executed.'],
              ['Text and JSON reports', 'Readable terminal output and a stable JSON schema, with documented exit codes for CI.'],
              ['Tested rule', 'One implemented rule with positive, negative and edge-case fixtures, and documented false positives and negatives.'],
            ].map(([t, d]) => (
              <li key={t} className="rounded-lg border border-ink-700 bg-ink-900 p-4">
                <h3 className="font-semibold text-white">{t}</h3>
                <p className="mt-1 text-sm text-slate-300">{d}</p>
              </li>
            ))}
          </ul>
          <p className="mt-4 text-sm"><Link className="text-accent underline" to="/features">See implemented and planned features</Link></p>
        </section>

        <section aria-labelledby="catalog">
          <h2 id="catalog" className="text-2xl font-bold text-white">Implemented security rules</h2>
          <ul className="mt-6 space-y-3">
            {implemented.map((r) => (
              <li key={r.id} className="rounded-lg border border-ink-700 bg-ink-900 p-4">
                <div className="flex flex-wrap items-center gap-2">
                  <Link to={`/rules/${r.id}`} className="font-mono font-semibold text-accent hover:underline">{r.id}</Link>
                  <span className="font-semibold text-white">{r.name}</span>
                  <StatusBadge status={r.status} />
                  <SeverityBadge severity={r.severity} />
                  <ConfidenceBadge confidence={r.confidence} />
                </div>
                <p className="mt-2 text-sm text-slate-300">{r.concern}</p>
              </li>
            ))}
          </ul>
          <p className="mt-4 text-sm text-slate-400">
            {plannedCount} further rule{plannedCount === 1 ? ' is' : 's are'} planned and not run by the scanner.{' '}
            <Link className="text-accent underline" to="/rules">Browse the full catalog</Link>.
          </p>
        </section>

        <section aria-labelledby="workflow">
          <h2 id="workflow" className="text-2xl font-bold text-white">Developer workflow</h2>
          <CodeBlock
            label="bash"
            code={`cargo build --release\n./target/release/sorobanshield scan path/to/contract\n./target/release/sorobanshield scan path/to/contract --format json --fail-on medium`}
          />
          <p className="text-sm text-slate-400">Build from source only: there is no published release yet.</p>
        </section>

        <section aria-labelledby="docs-entry">
          <h2 id="docs-entry" className="text-2xl font-bold text-white">Documentation</h2>
          <ul className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {[
              ['/docs/installation', 'Installation'],
              ['/docs/quickstart', 'Quickstart'],
              ['/docs/cli-reference', 'CLI reference'],
              ['/docs/limitations', 'Limitations'],
            ].map(([to, label]) => (
              <li key={to}><Link to={to} className="block rounded-lg border border-ink-700 bg-ink-900 p-4 font-semibold text-white hover:border-accent">{label}</Link></li>
            ))}
          </ul>
        </section>

        <section aria-labelledby="limits" className="rounded-lg border border-amber-500/40 bg-amber-500/5 p-6">
          <h2 id="limits" className="text-2xl font-bold text-white">Limitations and current status</h2>
          <ul className="mt-3 list-disc space-y-1 pl-6 text-slate-300">
            <li>Static analysis can miss vulnerabilities and report false positives.</li>
            <li>A scan with no findings is not proof that a contract is secure.</li>
            <li>SorobanShield has not been independently audited and is not a replacement for an audit.</li>
            <li>No release has been published. See the <Link className="text-accent underline" to="/roadmap">roadmap</Link> for status.</li>
          </ul>
        </section>

        <section aria-labelledby="oss">
          <h2 id="oss" className="text-2xl font-bold text-white">Open source</h2>
          <p className="mt-2 text-slate-300">
            Contributions of rules, fixtures, tests and documentation are welcome. Start with the{' '}
            <Link className="text-accent underline" to="/contributing">contribution guide</Link> or the{' '}
            <a className="text-accent underline" href={REPO_URL}>repository</a>.
          </p>
        </section>
      </div>
    </>
  )
}
