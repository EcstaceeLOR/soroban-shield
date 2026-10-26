import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, matchPath } from 'react-router-dom'
import { AppRoutes } from '../app/AppRoutes'
import { docPages } from '../lib/docs'
import { findRule, rules } from '../lib/rules'
import { ROUTE_PATTERNS } from '../lib/site'

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <AppRoutes />
    </MemoryRouter>,
  )
}

const concretePaths = ROUTE_PATTERNS.map((p) => p.replace(':ruleId', 'SS-001'))

describe('routes', () => {
  it.each(concretePaths)('renders a single h1 for %s', (path) => {
    renderAt(path)
    const h1 = screen.getAllByRole('heading', { level: 1 })
    expect(h1).toHaveLength(1)
    expect(h1[0].textContent?.trim().length).toBeGreaterThan(0)
  })

  it('renders the specified landing headline and CTAs', () => {
    renderAt('/')
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Find the risks. Understand the code. Strengthen your contracts.')
    expect(screen.getAllByRole('link', { name: 'Get Started' }).length).toBeGreaterThan(0)
    expect(screen.getByRole('link', { name: 'Explore Security Rules' })).toHaveAttribute('href', '/rules')
  })

  it('shows a 404 page for unknown routes', () => {
    renderAt('/does/not/exist')
    expect(screen.getByRole('heading', { level: 1, name: 'Page not found' })).toBeInTheDocument()
  })

  it('shows a 404 page for unknown docs pages', () => {
    renderAt('/docs/nope')
    expect(screen.getByRole('heading', { level: 1, name: 'Page not found' })).toBeInTheDocument()
  })

  it('every route pattern has a page, and every doc page is in the route table', () => {
    for (const d of docPages) expect(ROUTE_PATTERNS).toContain(`/docs/${d.slug}`)
    expect(ROUTE_PATTERNS.filter((p) => p.startsWith('/docs/')).length).toBe(docPages.length)
  })

  it('has no dead internal links on any page', () => {
    const bad: string[] = []
    for (const path of concretePaths) {
      const { container, unmount } = renderAt(path)
      for (const a of container.querySelectorAll('a[href^="/"]')) {
        const href = a.getAttribute('href')!.split('#')[0]
        const pattern = ROUTE_PATTERNS.find((p) => matchPath(p, href))
        const ruleOk = !pattern?.includes(':ruleId') || !!findRule(href.split('/').pop())
        if (!pattern || !ruleOk) bad.push(`${path} -> ${href}`)
      }
      unmount()
    }
    expect(bad).toEqual([])
  })
})

describe('rule catalog data', () => {
  it('is generated from the CLI and lists every shipped rule as implemented', () => {
    expect(rules.map((r) => [r.id, r.status])).toEqual([
      ['SS-001', 'implemented'],
      ['SS-002', 'implemented'],
      ['SS-003', 'implemented'],
      ['SS-004', 'implemented'],
      ['SS-005', 'implemented'],
      ['SS-006', 'implemented'],
    ])
  })

  it.each([
    ['SS-001', 'Authorization path review', 'severity: medium', 'confidence: low'],
    ['SS-002', 'Arithmetic boundary review', 'severity: low', 'confidence: low'],
    ['SS-003', 'External contract call review', 'severity: low', 'confidence: low'],
    ['SS-004', 'Error-path review', 'severity: info', 'confidence: low'],
    ['SS-005', 'State-transition test suggestions', 'severity: info', 'confidence: low'],
    ['SS-006', 'Dependency and toolchain diagnostics', 'severity: info', 'confidence: high'],
  ])('%s detail page shows %s with its severity, confidence and limitations', (id, name, sev, conf) => {
    renderAt(`/rules/${id}`)
    expect(screen.getByRole('heading', { level: 1, name })).toBeInTheDocument()
    expect(screen.getByText(sev)).toBeInTheDocument()
    expect(screen.getByText(conf)).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Known false positives' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Known false negatives' })).toBeInTheDocument()
    expect(screen.queryByRole('note')).not.toBeInTheDocument()
  })

  it('lists every rule as implemented on the catalog page', () => {
    renderAt('/rules')
    for (const r of rules) {
      expect(within(screen.getByTestId(`rule-${r.id}`)).getByTestId('status-badge')).toHaveTextContent('implemented')
    }
    expect(screen.queryByText(/not run by the scanner/i)).not.toBeInTheDocument()
  })

  it('SS-006 makes no vulnerability claim', () => {
    renderAt('/rules/SS-006')
    expect(screen.getByText(/does not consult any advisory database/i)).toBeInTheDocument()
  })

  it('filters by severity and status and shows an empty state', async () => {
    const user = userEvent.setup()
    renderAt('/rules')
    await user.selectOptions(screen.getByLabelText('Severity'), 'info')
    expect(screen.getByTestId('rule-SS-004')).toBeInTheDocument()
    expect(screen.getByTestId('rule-SS-006')).toBeInTheDocument()
    expect(screen.queryByTestId('rule-SS-001')).not.toBeInTheDocument()

    await user.selectOptions(screen.getByLabelText('Status'), 'planned')
    expect(screen.getByText('No rules match these filters.')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Reset filters' }))
    expect(screen.getByTestId('rule-SS-001')).toBeInTheDocument()
    expect(screen.getByTestId('rule-SS-006')).toBeInTheDocument()
  })

  it('shows a not-found state for unknown rule IDs', () => {
    renderAt('/rules/SS-999')
    expect(screen.getByRole('heading', { level: 1, name: 'Rule not found' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /back to the rule catalog/i })).toHaveAttribute('href', '/rules')
  })
})

describe('landing page honesty', () => {
  it('lists the implemented rules and mentions no planned rules when none exist', () => {
    renderAt('/')
    expect(screen.getAllByText('SS-001').length).toBeGreaterThan(0)
    expect(screen.queryByText(/planned and not run by the scanner/i)).not.toBeInTheDocument()
  })

  it('uses real fixture output in the terminal preview and states that a clean scan proves nothing', () => {
    renderAt('/')
    expect(screen.getByText(/Real output of the CLI/i)).toBeInTheDocument()
    expect(screen.getByText(/SS-001 Authorization path review \(severity: medium, confidence: low\)/)).toBeInTheDocument()
    expect(screen.getByText(/not proof that a contract is secure/i)).toBeInTheDocument()
  })

  it('does not claim to be audited, safe or released', () => {
    renderAt('/')
    expect(document.body.textContent).not.toMatch(/\b(fully secure|guarantees? (safety|security)|audited by)\b/i)
    expect(document.body.textContent).toMatch(/not been independently audited/i)
  })
})

describe('documentation pages', () => {
  it('documents the supported configuration file and its strict validation', () => {
    renderAt('/docs/configuration')
    expect(screen.getAllByText(/sorobanshield\.toml/).length).toBeGreaterThan(0)
    expect(screen.getByText(/Unknown keys and sections are rejected/i)).toBeInTheDocument()
  })

  it('documents all four exit codes in the CLI reference', () => {
    renderAt('/docs/cli-reference')
    for (const c of ['0', '1', '2', '3']) {
      expect(screen.getAllByRole('cell', { name: c }).length).toBeGreaterThan(0)
    }
  })

  it('marks CI integration as untested', () => {
    renderAt('/docs/ci-integration')
    expect(screen.getByText(/Status: untested/)).toBeInTheDocument()
  })

  it('docs sidebar links to every doc page', () => {
    renderAt('/docs')
    const nav = within(screen.getByRole('navigation', { name: 'Documentation' }))
    for (const d of docPages) {
      expect(nav.getByRole('link', { name: d.title })).toHaveAttribute('href', `/docs/${d.slug}`)
    }
  })

  it('marks the current docs page in the sidebar', () => {
    renderAt('/docs/reports')
    const nav = within(screen.getByRole('navigation', { name: 'Documentation' }))
    expect(nav.getByRole('link', { name: 'Reports' })).toHaveAttribute('aria-current', 'page')
  })
})

describe('responsive navigation', () => {
  it('toggles the mobile menu with aria-expanded', async () => {
    const user = userEvent.setup()
    renderAt('/')
    const btn = screen.getByRole('button', { name: /toggle navigation menu/i })
    expect(btn).toHaveAttribute('aria-expanded', 'false')
    expect(screen.queryByRole('navigation', { name: 'Mobile' })).not.toBeInTheDocument()
    await user.click(btn)
    expect(btn).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByRole('navigation', { name: 'Mobile' })).toBeInTheDocument()
    await user.click(btn)
    expect(btn).toHaveAttribute('aria-expanded', 'false')
  })

  it('is operable by keyboard and closes on Escape, returning focus to the button', async () => {
    const user = userEvent.setup()
    renderAt('/')
    const btn = screen.getByRole('button', { name: /toggle navigation menu/i })
    btn.focus()
    await user.keyboard('{Enter}')
    expect(btn).toHaveAttribute('aria-expanded', 'true')
    await user.keyboard('{Escape}')
    expect(btn).toHaveAttribute('aria-expanded', 'false')
    expect(btn).toHaveFocus()
    await user.keyboard(' ')
    expect(btn).toHaveAttribute('aria-expanded', 'true')
  })

  it('closes the mobile menu after navigating', async () => {
    const user = userEvent.setup()
    renderAt('/')
    const btn = screen.getByRole('button', { name: /toggle navigation menu/i })
    await user.click(btn)
    const mobile = within(screen.getByRole('navigation', { name: 'Mobile' }))
    await user.click(mobile.getByRole('link', { name: 'Roadmap' }))
    expect(screen.getByRole('heading', { level: 1, name: 'Roadmap' })).toBeInTheDocument()
    expect(btn).toHaveAttribute('aria-expanded', 'false')
  })

  it('provides a skip link and footer links to repository, roadmap, contributing and security', () => {
    renderAt('/about')
    expect(screen.getByRole('link', { name: 'Skip to content' })).toHaveAttribute('href', '#main')
    const footer = within(screen.getByRole('contentinfo'))
    expect(footer.getByRole('link', { name: 'Roadmap' })).toBeInTheDocument()
    expect(footer.getByRole('link', { name: 'Contributing' })).toBeInTheDocument()
    expect(footer.getByRole('link', { name: 'Security policy' })).toHaveAttribute('href', expect.stringContaining('SECURITY.md'))
    expect(footer.getByRole('link', { name: 'Repository' })).toHaveAttribute('href', 'https://github.com/sorobanshield/soroban-shield')
  })
})

describe('roadmap honesty', () => {
  it('does not mark unimplemented work as done', () => {
    renderAt('/roadmap')
    const item = (title: string) => screen.getByRole('heading', { name: title }).closest('li')!
    expect(within(item('Baseline support')).getByText('done')).toBeInTheDocument()
    expect(within(item('SARIF output')).getByText('done')).toBeInTheDocument()
    expect(within(item('SARIF output')).getByText(/not in CI/i)).toBeInTheDocument()
    expect(within(item('Configuration file')).getByText('done')).toBeInTheDocument()
    expect(within(item('First tagged release')).getByText('planned')).toBeInTheDocument()
    expect(within(item('JSON report and exit codes')).getByText('done')).toBeInTheDocument()
  })
})
