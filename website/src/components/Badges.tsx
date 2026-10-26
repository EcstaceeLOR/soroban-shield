import type { Confidence, RuleStatus, Severity } from '../lib/rules'

const base = 'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium'

const statusStyle: Record<RuleStatus, string> = {
  implemented: 'border-emerald-500/50 bg-emerald-500/10 text-emerald-300',
  experimental: 'border-amber-500/50 bg-amber-500/10 text-amber-300',
  planned: 'border-slate-500/60 bg-slate-500/10 text-slate-300',
  deprecated: 'border-rose-500/50 bg-rose-500/10 text-rose-300',
}

export function StatusBadge({ status }: { status: RuleStatus }) {
  return (
    <span className={`${base} ${statusStyle[status]}`} data-testid="status-badge">
      {status}
    </span>
  )
}

const severityStyle: Record<Severity, string> = {
  info: 'border-sky-500/50 text-sky-300',
  low: 'border-lime-500/50 text-lime-300',
  medium: 'border-amber-500/50 text-amber-300',
  high: 'border-orange-500/50 text-orange-300',
  critical: 'border-rose-500/50 text-rose-300',
}

export function SeverityBadge({ severity }: { severity: Severity | null }) {
  if (!severity) return <span className={`${base} border-slate-600 text-slate-400`}>severity: not assigned</span>
  return <span className={`${base} ${severityStyle[severity]}`}>severity: {severity}</span>
}

export function ConfidenceBadge({ confidence }: { confidence: Confidence | null }) {
  if (!confidence) return <span className={`${base} border-slate-600 text-slate-400`}>confidence: not assigned</span>
  return <span className={`${base} border-slate-500 text-slate-200`}>confidence: {confidence}</span>
}
