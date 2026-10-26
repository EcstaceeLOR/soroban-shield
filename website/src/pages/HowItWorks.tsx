import { Link } from 'react-router-dom'
import { Page, Section } from '../components/Page'

export default function HowItWorks() {
  return (
    <Page title="How it works" lead="A scan is a local, read-only pass over your Rust source.">
      <Section title="1. Project discovery">
        <p>The scan starts at the path you give it, which must contain a <code>Cargo.toml</code>. Every manifest is parsed; at least one must declare <code>soroban-sdk</code>. Rust files are collected, skipping <code>target</code>, <code>node_modules</code>, <code>.git</code>, <code>vendor</code>, <code>dist</code> and <code>build</code>.</p>
        <p>If the path is missing, the manifest is malformed, or the project is not a Soroban project, the scan stops with a clear error and exit code 3. It is never reported as clean.</p>
      </Section>
      <Section title="2. Parsing">
        <p>Each file is parsed into a syntax tree. Nothing is compiled and no build script or project code is run. A file that cannot be read or parsed is recorded as an error and makes the whole scan incomplete (exit code 3).</p>
      </Section>
      <Section title="3. Rules">
        <p>Each implemented rule inspects the syntax tree. Today that is <Link className="text-accent underline" to="/rules/SS-001">SS-001</Link>, which looks for public <code>#[contractimpl]</code> functions that write storage without a visible <code>require_auth</code>. Planned rules are never run.</p>
      </Section>
      <Section title="4. Findings and reports">
        <p>Findings carry a rule ID, severity (potential impact), confidence (how sure the rule is), location, message, remediation and limitations. They are sorted by file and line, then printed as text or JSON.</p>
      </Section>
      <Section title="5. Exit code">
        <p>Exit 0 when nothing reached <code>--fail-on</code>, 1 when something did, 2 for invalid usage and 3 when the scan could not be completed. See the <Link className="text-accent underline" to="/docs/cli-reference">CLI reference</Link>.</p>
      </Section>
      <Section title="Using the results">
        <p>Treat findings as prompts for review and testing. A clean scan means only that the implemented checks found nothing; read the <Link className="text-accent underline" to="/docs/limitations">limitations</Link>.</p>
      </Section>
    </Page>
  )
}
