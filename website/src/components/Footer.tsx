import { Link } from 'react-router-dom'
import { REPO_URL } from '../lib/site'

export function Footer() {
  return (
    <footer className="mt-16 border-t border-ink-700 bg-ink-900">
      <div className="mx-auto grid max-w-6xl gap-8 px-4 py-10 text-sm md:grid-cols-3">
        <div>
          <p className="font-mono font-bold text-white">SorobanShield</p>
          <p className="mt-2 max-w-xs text-slate-400">
            An early-stage, open-source review aid for Soroban contracts. It is not an audit and a clean scan is not proof of security.
          </p>
        </div>
        <nav aria-label="Project">
          <p className="font-semibold text-white">Project</p>
          <ul className="mt-2 space-y-1 text-slate-300">
            <li><Link to="/roadmap">Roadmap</Link></li>
            <li><Link to="/contributing">Contributing</Link></li>
            <li><Link to="/about">About</Link></li>
            <li><a href={REPO_URL}>Repository</a></li>
            <li><a href={`${REPO_URL}/issues`}>Issues</a></li>
          </ul>
        </nav>
        <nav aria-label="Resources">
          <p className="font-semibold text-white">Resources</p>
          <ul className="mt-2 space-y-1 text-slate-300">
            <li><Link to="/docs">Documentation</Link></li>
            <li><Link to="/rules">Security rules</Link></li>
            <li><Link to="/docs/limitations">Limitations</Link></li>
            <li><Link to="/docs/security-model">Security model</Link></li>
            <li><a href={`${REPO_URL}/blob/main/SECURITY.md`}>Security policy</a></li>
            <li><a href={`${REPO_URL}/blob/main/LICENSE`}>License</a></li>
          </ul>
        </nav>
      </div>
    </footer>
  )
}
