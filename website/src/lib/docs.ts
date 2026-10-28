import installation from '../../../docs/installation.md?raw'
import quickstart from '../../../docs/quickstart.md?raw'
import cliReference from '../../../docs/cli-reference.md?raw'
import configuration from '../../../docs/configuration.md?raw'
import reports from '../../../docs/reports.md?raw'
import rules from '../../../docs/rule-authoring.md?raw'
import ciIntegration from '../../../docs/ci-integration.md?raw'
import securityModel from '../../../docs/threat-model.md?raw'
import limitations from '../../../docs/limitations.md?raw'

// The Markdown sources live in the repository's docs/ directory, which is the single
// canonical copy shared by GitHub and this site.

export interface DocPage {
  slug: string
  title: string
  summary: string
  body: string
}

export const docPages: DocPage[] = [
  { slug: 'installation', title: 'Installation', summary: 'Build from source and the supported project layout.', body: installation },
  { slug: 'quickstart', title: 'Quickstart', summary: 'Run a first scan against repository fixtures.', body: quickstart },
  { slug: 'cli-reference', title: 'CLI reference', summary: 'Commands, options and exit codes.', body: cliReference },
  { slug: 'configuration', title: 'Configuration', summary: 'What can be configured today (flags only).', body: configuration },
  { slug: 'reports', title: 'Reports', summary: 'Text, JSON and SARIF output formats.', body: reports },
  { slug: 'rules', title: 'Rule authoring', summary: 'How to implement, test and document a rule.', body: rules },
  { slug: 'ci-integration', title: 'CI integration', summary: 'Exit codes in CI, with untested illustrative steps.', body: ciIntegration },
  { slug: 'security-model', title: 'Security model', summary: 'Privacy, trust boundaries and disclosure.', body: securityModel },
  { slug: 'limitations', title: 'Limitations', summary: 'False positives, false negatives and scope.', body: limitations },
]

// Maps a repository docs/ file name to the website route slug, so relative
// links between documents keep working on the site.
export const docFileSlugs: Record<string, string> = {
  'installation.md': 'installation',
  'quickstart.md': 'quickstart',
  'cli-reference.md': 'cli-reference',
  'configuration.md': 'configuration',
  'reports.md': 'reports',
  'rule-authoring.md': 'rules',
  'ci-integration.md': 'ci-integration',
  'threat-model.md': 'security-model',
  'limitations.md': 'limitations',
}
