import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { ConfidenceBadge, SeverityBadge, StatusBadge } from '../components/Badges'
import { rules, type RuleStatus, type Severity } from '../lib/rules'

type StatusFilter = RuleStatus | 'all'
type SeverityFilter = Severity | 'none' | 'all'

const selectClass = 'mt-1 block rounded border border-ink-600 bg-ink-900 px-3 py-2 text-sm text-white'

export default function Rules() {
  const [status, setStatus] = useState<StatusFilter>('all')
  const [severity, setSeverity] = useState<SeverityFilter>('all')

  const shown = useMemo(
    () =>
      rules.filter(
        (r) =>
          (status === 'all' || r.status === status) &&
          (severity === 'all' || (severity === 'none' ? r.severity === null : r.severity === severity)),
      ),
    [status, severity],
  )

  return (
    <div className="mx-auto max-w-5xl px-4 py-12">
      <h1 className="text-3xl font-bold text-white md:text-4xl">Security rules</h1>
      <p className="mt-3 text-lg text-slate-300">
        The catalog is generated from the CLI. Only rules marked <strong>implemented</strong> run during a scan; planned rules are listed for transparency and make no detection claims.
      </p>

      <form className="mt-8 flex flex-wrap items-end gap-4" onSubmit={(e) => e.preventDefault()} aria-label="Filter rules">
        <div>
          <label htmlFor="status-filter" className="text-sm text-slate-300">Status</label>
          <select id="status-filter" className={selectClass} value={status} onChange={(e) => setStatus(e.target.value as StatusFilter)}>
            <option value="all">All statuses</option>
            <option value="implemented">Implemented</option>
            <option value="experimental">Experimental</option>
            <option value="planned">Planned</option>
            <option value="deprecated">Deprecated</option>
          </select>
        </div>
        <div>
          <label htmlFor="severity-filter" className="text-sm text-slate-300">Severity</label>
          <select id="severity-filter" className={selectClass} value={severity} onChange={(e) => setSeverity(e.target.value as SeverityFilter)}>
            <option value="all">All severities</option>
            <option value="info">Info</option>
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
            <option value="critical">Critical</option>
            <option value="none">Not assigned</option>
          </select>
        </div>
      </form>

      <p className="mt-4 text-sm text-slate-400" role="status" aria-live="polite">
        Showing {shown.length} of {rules.length} rules
      </p>

      {shown.length === 0 ? (
        <div className="mt-6 rounded-lg border border-dashed border-ink-600 p-8 text-center">
          <p className="font-semibold text-white">No rules match these filters.</p>
          <button type="button" className="mt-3 rounded border border-ink-600 px-3 py-1.5 text-sm text-accent hover:bg-ink-800" onClick={() => { setStatus('all'); setSeverity('all') }}>
            Reset filters
          </button>
        </div>
      ) : (
        <ul className="mt-6 space-y-3">
          {shown.map((r) => (
            <li key={r.id} className="rounded-lg border border-ink-700 bg-ink-900 p-4" data-testid={`rule-${r.id}`}>
              <div className="flex flex-wrap items-center gap-2">
                <Link to={`/rules/${r.id}`} className="font-mono font-semibold text-accent hover:underline">{r.id}</Link>
                <span className="font-semibold text-white">{r.name}</span>
                <StatusBadge status={r.status} />
                {r.status !== 'planned' && <SeverityBadge severity={r.severity} />}
                {r.status !== 'planned' && <ConfidenceBadge confidence={r.confidence} />}
              </div>
              <p className="mt-2 text-sm text-slate-300">{r.concern}</p>
              {r.status === 'planned' && <p className="mt-1 text-xs text-slate-400">Planned: not run by the scanner. No severity or confidence assigned.</p>}
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
