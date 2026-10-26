import installation from '../content/docs/installation.md?raw'
import quickstart from '../content/docs/quickstart.md?raw'
import cliReference from '../content/docs/cli-reference.md?raw'
import configuration from '../content/docs/configuration.md?raw'
import reports from '../content/docs/reports.md?raw'
import rules from '../content/docs/rules.md?raw'
import ciIntegration from '../content/docs/ci-integration.md?raw'
import securityModel from '../content/docs/security-model.md?raw'
import limitations from '../content/docs/limitations.md?raw'

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
  { slug: 'reports', title: 'Reports', summary: 'Text and JSON output formats.', body: reports },
  { slug: 'rules', title: 'Rule authoring', summary: 'How to implement, test and document a rule.', body: rules },
  { slug: 'ci-integration', title: 'CI integration', summary: 'Exit codes in CI, with untested illustrative steps.', body: ciIntegration },
  { slug: 'security-model', title: 'Security model', summary: 'Privacy, trust boundaries and disclosure.', body: securityModel },
  { slug: 'limitations', title: 'Limitations', summary: 'False positives, false negatives and scope.', body: limitations },
]
