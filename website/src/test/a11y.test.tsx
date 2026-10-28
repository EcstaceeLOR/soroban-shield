import axe from 'axe-core'
import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { AppRoutes } from '../app/AppRoutes'
import { ROUTE_PATTERNS } from '../lib/site'

// jsdom has no layout engine, so colour-contrast cannot be computed here;
// it is excluded rather than silently reported as passing.
const paths = [...ROUTE_PATTERNS.map((p) => p.replace(':ruleId', 'SS-001')), '/rules/SS-999', '/no-such-page']

describe('automated accessibility checks (axe-core, no layout)', () => {
  it.each(paths)('has no axe violations on %s', async (path) => {
    const { container } = render(
      <MemoryRouter initialEntries={[path]}>
        <AppRoutes />
      </MemoryRouter>,
    )
    await screen.findAllByRole('heading', { level: 1 })
    const results = await axe.run(container, {
      rules: { 'color-contrast': { enabled: false }, region: { enabled: false } },
    })
    const summary = results.violations.map((v) => `${v.id}: ${v.help} (${v.nodes.length} node(s))`)
    expect(summary).toEqual([])
  })
})
