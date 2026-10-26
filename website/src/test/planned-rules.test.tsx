import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { vi } from 'vitest'
import { AppRoutes } from '../app/AppRoutes'

// The shipped catalog has no planned rules, but the UI supports the status.
// These tests inject one synthetic planned rule to keep that behavior honest.
vi.mock('../lib/rules', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../lib/rules')>()
  const planned = {
    id: 'SS-900',
    name: 'Synthetic planned rule',
    status: 'planned' as const,
    severity: null,
    confidence: null,
    concern: 'Synthetic concern.',
    detection: 'Synthetic approach.',
    severity_rationale: 'Not assigned.',
    remediation: 'Not applicable.',
    false_positives: [],
    false_negatives: [],
    references: [],
    fixtures: [],
  }
  const all = [...actual.rules, planned]
  return {
    ...actual,
    rules: all,
    findRule: (id: string | undefined) => all.find((r) => r.id.toLowerCase() === (id ?? '').toLowerCase()),
    implementedRules: () => all.filter((r) => r.status === 'implemented'),
  }
})

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <AppRoutes />
    </MemoryRouter>,
  )
}

describe('planned rules in the UI', () => {
  it('labels a planned rule on the catalog without severity or confidence', () => {
    renderAt('/rules')
    const card = within(screen.getByTestId('rule-SS-900'))
    expect(card.getByTestId('status-badge')).toHaveTextContent('planned')
    expect(card.queryByText(/severity:/)).not.toBeInTheDocument()
    expect(card.getByText(/not run by the scanner/i)).toBeInTheDocument()
    expect(within(screen.getByTestId('rule-SS-001')).getByTestId('status-badge')).toHaveTextContent('implemented')
  })

  it('filters planned rules by status and by "not assigned" severity', async () => {
    const user = userEvent.setup()
    renderAt('/rules')
    await user.selectOptions(screen.getByLabelText('Status'), 'planned')
    expect(screen.getByTestId('rule-SS-900')).toBeInTheDocument()
    expect(screen.queryByTestId('rule-SS-001')).not.toBeInTheDocument()

    await user.selectOptions(screen.getByLabelText('Status'), 'all')
    await user.selectOptions(screen.getByLabelText('Severity'), 'none')
    expect(screen.getByTestId('rule-SS-900')).toBeInTheDocument()
    expect(screen.queryByTestId('rule-SS-001')).not.toBeInTheDocument()
  })

  it('labels a planned rule page as not implemented, with no severity claim', () => {
    renderAt('/rules/SS-900')
    expect(screen.getByRole('note')).toHaveTextContent(/planned.*not implemented/i)
    expect(screen.getByText('severity: not assigned')).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'Known false positives' })).not.toBeInTheDocument()
  })

  it('does not present a planned rule as active on the landing page', () => {
    renderAt('/')
    expect(screen.queryByText('SS-900')).not.toBeInTheDocument()
    expect(screen.queryByText('Synthetic planned rule')).not.toBeInTheDocument()
    expect(screen.getByText(/1 further rule is planned and not run by the scanner/i)).toBeInTheDocument()
  })
})
