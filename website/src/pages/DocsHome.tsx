import { Link } from 'react-router-dom'
import { docPages } from '../lib/docs'

export default function DocsHome() {
  return (
    <div>
      <h1 className="text-3xl font-bold text-white">Documentation</h1>
      <p className="mt-3 text-slate-300">Everything here describes the CLI as it exists today. Planned features are labelled planned.</p>
      <ul className="mt-6 grid gap-3 sm:grid-cols-2">
        {docPages.map((d) => (
          <li key={d.slug}>
            <Link to={`/docs/${d.slug}`} className="block h-full rounded-lg border border-ink-700 bg-ink-900 p-4 hover:border-accent">
              <span className="font-semibold text-white">{d.title}</span>
              <span className="mt-1 block text-sm text-slate-400">{d.summary}</span>
            </Link>
          </li>
        ))}
      </ul>
      <p className="mt-6 text-sm text-slate-300">Looking for a specific check? See the <Link className="text-accent underline" to="/rules">rule catalog</Link>.</p>
    </div>
  )
}
