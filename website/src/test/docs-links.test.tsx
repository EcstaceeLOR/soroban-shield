import { render, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { AppRoutes } from '../app/AppRoutes'
import { docFileSlugs, docPages } from '../lib/docs'
import { REPO_URL } from '../lib/site'

// The docs are authored for GitHub and shared with the site, so relative links
// must work in both places.
const sources = import.meta.glob('../../../docs/*.md', { query: '?raw', import: 'default', eager: true }) as Record<string, string>

const relativeLinks = Object.entries(sources).flatMap(([path, text]) =>
  [...text.matchAll(/\]\(([^)\s]+)\)/g)]
    .map((m) => m[1])
    .filter((href) => !/^(https?:|#|mailto:)/.test(href))
    .map((href) => ({ file: path.split('/').pop()!, href })),
)

describe('shared docs', () => {
  it('serves every site doc page from a file in docs/', () => {
    const files = Object.keys(sources).map((p) => p.split('/').pop()!)
    for (const file of Object.keys(docFileSlugs)) expect(files).toContain(file)
    expect(new Set(Object.values(docFileSlugs))).toEqual(new Set(docPages.map((d) => d.slug)))
  })

  it('only uses relative links that map to a site page or a repository file', () => {
    expect(relativeLinks.length).toBeGreaterThan(0)
    for (const { file, href } of relativeLinks) {
      const [target] = href.split('#')
      const ok = docFileSlugs[target] !== undefined || target.startsWith('../')
      expect(ok, `${file} links to ${href}`).toBe(true)
    }
  })

  it('rewrites a doc-to-doc link to a site route and a ../ link to the repository', () => {
    // Scoped to the rendered document body: the sidebar has links of the same name.
    const docBody = (path: string) => {
      const { container, unmount } = render(
        <MemoryRouter initialEntries={[path]}>
          <AppRoutes />
        </MemoryRouter>,
      )
      return { body: within(container.querySelector('.doc') as HTMLElement), unmount }
    }

    const cli = docBody('/docs/cli-reference')
    expect(cli.body.getByRole('link', { name: 'Configuration' })).toHaveAttribute('href', '/docs/configuration')
    cli.unmount()

    const model = docBody('/docs/security-model')
    expect(model.body.getByRole('link', { name: 'SECURITY.md' })).toHaveAttribute('href', `${REPO_URL}/blob/main/SECURITY.md`)
  })
})
