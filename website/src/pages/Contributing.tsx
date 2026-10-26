import { Link } from 'react-router-dom'
import { CodeBlock } from '../components/CodeBlock'
import { Page, Section } from '../components/Page'
import { REPO_URL } from '../lib/site'

export default function Contributing() {
  return (
    <Page title="Contributing" lead="Rules, fixtures, tests, documentation and bug reports are all welcome.">
      <Section title="Set up">
        <CodeBlock label="bash" code={`git clone ${REPO_URL}.git\ncd soroban-shield\ncargo build\ncargo test --workspace`} />
      </Section>
      <Section title="Checks to run before a pull request">
        <CodeBlock label="bash" code={`cargo fmt --all -- --check\ncargo clippy --workspace --all-targets -- -D warnings\ncargo test --workspace\n\ncd website\nnpm ci\nnpm run lint\nnpm run typecheck\nnpm run test -- --run\nnpm run build`} />
      </Section>
      <Section title="Workflow">
        <ol className="list-decimal space-y-1 pl-6">
          <li>Open or pick an issue; discuss substantial design changes first.</li>
          <li>Keep pull requests focused and add tests for behaviour changes.</li>
          <li>Update documentation when CLI or rule behaviour changes.</li>
          <li>Record limitations and likely false positives. Do not overstate certainty.</li>
          <li>Never commit secrets or private source code.</li>
        </ol>
      </Section>
      <Section title="Adding a rule">
        <p>A rule needs a stable ID, syntax-aware detection, positive, negative and edge-case fixtures, documented limitations, and registration in the rule list. See <Link className="text-accent underline" to="/docs/rules">rule authoring</Link>.</p>
      </Section>
      <Section title="Reporting security issues">
        <p>Report vulnerabilities in SorobanShield privately following <a className="text-accent underline" href={`${REPO_URL}/blob/main/SECURITY.md`}>SECURITY.md</a>, not in public issues.</p>
      </Section>
    </Page>
  )
}
