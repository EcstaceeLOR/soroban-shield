import data from '../data/rules.generated.json'

export type RuleStatus = 'planned' | 'experimental' | 'implemented' | 'deprecated'
export type Severity = 'info' | 'low' | 'medium' | 'high' | 'critical'
export type Confidence = 'low' | 'medium' | 'high'

export interface Rule {
  id: string
  name: string
  status: RuleStatus
  severity: Severity | null
  confidence: Confidence | null
  concern: string
  detection: string
  severity_rationale: string
  remediation: string
  false_positives: string[]
  false_negatives: string[]
  references: string[]
  fixtures: string[]
}

export const rules = data as Rule[]

export const findRule = (id: string | undefined): Rule | undefined =>
  rules.find((r) => r.id.toLowerCase() === (id ?? '').toLowerCase())

export const implementedRules = () => rules.filter((r) => r.status === 'implemented')
