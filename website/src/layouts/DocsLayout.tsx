import { NavLink, Outlet } from 'react-router-dom'
import { docPages } from '../lib/docs'

export function DocsLayout() {
  const cls = ({ isActive }: { isActive: boolean }) =>
    `block whitespace-nowrap rounded px-3 py-1.5 text-sm md:whitespace-normal ${isActive ? 'bg-ink-700 text-white' : 'text-slate-300 hover:bg-ink-800'}`
  return (
    <div className="mx-auto grid max-w-6xl gap-8 px-4 py-10 md:grid-cols-[14rem_1fr]">
      <nav aria-label="Documentation" className="md:sticky md:top-20 md:self-start">
        <p className="mb-2 hidden px-3 text-xs md:block font-semibold uppercase tracking-wide text-slate-500">Documentation</p>
        <ul className="flex gap-1 overflow-x-auto pb-2 md:block md:space-y-0.5 md:overflow-visible md:pb-0">
          <li><NavLink to="/docs" end className={cls}>Overview</NavLink></li>
          {docPages.map((d) => (
            <li key={d.slug}>
              <NavLink to={`/docs/${d.slug}`} className={cls}>{d.title}</NavLink>
            </li>
          ))}
        </ul>
      </nav>
      <div className="min-w-0 max-w-3xl">
        <Outlet />
      </div>
    </div>
  )
}
