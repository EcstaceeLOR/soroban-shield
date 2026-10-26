import { Link, useParams } from 'react-router-dom'
import { ConfidenceBadge, SeverityBadge, StatusBadge } from '../components/Badges'
import { CodeBlock } from '../components/CodeBlock'
import { findRule } from '../lib/rules'
import { REPO_URL } from '../lib/site'

function List({ title, items }: { title: string; items: string[] }) {
  return (
    <section>
      <h2 className="mb-2 text-xl font-semibold text-white">{title}</h2>
      {items.length === 0 ? <p className="text-sm text-slate-400">None documented.</p> : (
        <ul className="list-disc space-y-1 pl-6 text-slate-300">{items.map((i) => <li key={i}>{i}</li>)}</ul>
      )}
    </section>
  )
}

export default function RuleDetail() {
  const { ruleId } = useParams()
  const rule = findRule(ruleId)

  if (!rule) {
    return (
      <div className="mx-auto max-w-3xl px-4 py-16">
        <h1 className="text-3xl font-bold text-white">Rule not found</h1>
        <p className="mt-3 text-slate-300">There is no rule with the ID “{ruleId}”.</p>
        <p className="mt-4"><Link className="text-accent underline" to="/rules">Back to the rule catalog</Link></p>
      </div>
    )
  }

  const planned = rule.status === 'planned'
  return (
    <div className="mx-auto max-w-3xl space-y-8 px-4 py-12">
      <div>
        <p className="font-mono text-accent">{rule.id}</p>
        <h1 className="mt-1 text-3xl font-bold text-white">{rule.name}</h1>
        <div className="mt-3 flex flex-wrap gap-2">
          <StatusBadge status={rule.status} />
          <SeverityBadge severity={rule.severity} />
          <ConfidenceBadge confidence={rule.confidence} />
        </div>
      </div>

      {planned && (
        <p role="note" className="rounded-lg border border-slate-500/50 bg-slate-500/10 p-4 text-slate-200">
          This rule is <strong>planned</strong>. It is not implemented and the scanner does not run it, so a scan will never report it.
        </p>
      )}

      <section><h2 className="mb-2 text-xl font-semibold text-white">Concern</h2><p className="leading-7 text-slate-300">{rule.concern}</p></section>
      <section><h2 className="mb-2 text-xl font-semibold text-white">{planned ? 'Planned approach' : 'Detection method'}</h2><p className="leading-7 text-slate-300">{rule.detection}</p></section>
      <section><h2 className="mb-2 text-xl font-semibold text-white">Severity and confidence</h2><p className="leading-7 text-slate-300">{rule.severity_rationale}</p></section>
      {!planned && <section><h2 className="mb-2 text-xl font-semibold text-white">Remediation and investigation</h2><p className="leading-7 text-slate-300">{rule.remediation}</p></section>}
      {!planned && <List title="Known false positives" items={rule.false_positives} />}
      {!planned && <List title="Known false negatives" items={rule.false_negatives} />}
      {rule.fixtures.length > 0 && (
        <section>
          <h2 className="mb-2 text-xl font-semibold text-white">Fixtures</h2>
          <ul className="list-disc space-y-1 pl-6">
            {rule.fixtures.map((f) => <li key={f}><a className="font-mono text-sm text-accent underline" href={`${REPO_URL}/tree/main/${f}`}>{f}</a></li>)}
          </ul>
        </section>
      )}
      {rule.references.length > 0 && (
        <section>
          <h2 className="mb-2 text-xl font-semibold text-white">References</h2>
          <ul className="list-disc space-y-1 pl-6">{rule.references.map((r) => <li key={r}><a className="break-all text-accent underline" href={r}>{r}</a></li>)}</ul>
        </section>
      )}
      {!planned && <section><h2 className="mb-2 text-xl font-semibold text-white">Try it</h2><CodeBlock label="bash" code={`sorobanshield rules explain ${rule.id}`} /></section>}
      <p><Link className="text-accent underline" to="/rules">All rules</Link></p>
    </div>
  )
}
