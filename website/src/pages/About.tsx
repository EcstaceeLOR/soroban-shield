import { Link } from 'react-router-dom'
import { Page, Section } from '../components/Page'
import { REPO_URL } from '../lib/site'

export default function About() {
  return (
    <Page title="About" lead="A transparent, developer-friendly review aid for Soroban smart contracts.">
      <Section title="Mission">
        <p>Make some parts of contract review repeatable, explain why a pattern deserves attention, and encourage better tests, while being honest about what static analysis can and cannot do.</p>
      </Section>
      <Section title="Principles">
        <ul className="list-disc space-y-1 pl-6">
          <li>Few, well-tested checks over many weak ones.</li>
          <li>Severity and confidence are reported separately.</li>
          <li>Every rule documents its false positives and false negatives.</li>
          <li>Local by default: no source upload and no telemetry.</li>
          <li>No claims of safety, audits or guarantees.</li>
        </ul>
      </Section>
      <Section title="Open source">
        <p>The project is developed in the open at <a className="text-accent underline" href={REPO_URL}>{REPO_URL.replace('https://', '')}</a>. See <Link className="text-accent underline" to="/contributing">how to contribute</Link> and the <Link className="text-accent underline" to="/roadmap">roadmap</Link>.</p>
      </Section>
      <Section title="Status">
        <p>SorobanShield is early-stage software. It has not been independently audited and no release has been published.</p>
      </Section>
    </Page>
  )
}
