export const REPO_URL = 'https://github.com/sorobanshield/soroban-shield'

export const ROUTE_PATTERNS = [
  '/',
  '/features',
  '/how-it-works',
  '/rules',
  '/rules/:ruleId',
  '/docs',
  '/docs/installation',
  '/docs/quickstart',
  '/docs/cli-reference',
  '/docs/configuration',
  '/docs/reports',
  '/docs/rules',
  '/docs/ci-integration',
  '/docs/security-model',
  '/docs/limitations',
  '/contributing',
  '/roadmap',
  '/about',
] as const

export const NAV_LINKS = [
  { to: '/features', label: 'Features' },
  { to: '/how-it-works', label: 'How it works' },
  { to: '/rules', label: 'Security rules' },
  { to: '/docs', label: 'Docs' },
  { to: '/contributing', label: 'Contributing' },
  { to: '/roadmap', label: 'Roadmap' },
]
