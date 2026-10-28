import { useEffect, useRef, useState } from 'react'
import { Link, NavLink, useLocation } from 'react-router-dom'
import { NAV_LINKS, REPO_URL } from '../lib/site'

const linkClass = ({ isActive }: { isActive: boolean }) =>
  `rounded border-b-2 px-2 py-1 text-sm transition-colors ${isActive ? 'border-accent text-white' : 'border-transparent text-slate-300 hover:text-white'}`

export function Header() {
  const { pathname } = useLocation()
  const [openAt, setOpenAt] = useState<string | null>(null)
  const open = openAt === pathname
  const button = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setOpenAt(null)
        button.current?.focus()
      }
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open])

  return (
    <header className="sticky top-0 z-40 border-b border-ink-700 bg-ink-950/90 backdrop-blur">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3">
        <Link to="/" className="flex items-center gap-2 font-mono text-lg font-bold text-white">
          <span aria-hidden="true" className="text-accent">▲</span> SorobanShield
        </Link>
        <nav aria-label="Primary" className="hidden items-center gap-2 md:flex">
          {NAV_LINKS.map((l) => (
            <NavLink key={l.to} to={l.to} className={linkClass}>
              {l.label}
            </NavLink>
          ))}
          <a href={REPO_URL} className="rounded px-2 py-1 text-sm text-slate-300 hover:text-white">
            GitHub
          </a>
          <Link to="/docs/quickstart" className="ml-2 rounded-md bg-accent px-3 py-1.5 text-sm font-semibold text-ink-950 transition-colors hover:bg-accent-dim">
            Get Started
          </Link>
        </nav>
        <button
          ref={button}
          type="button"
          className="rounded border border-ink-600 px-3 py-1.5 text-sm text-slate-200 md:hidden"
          aria-expanded={open}
          aria-controls="mobile-nav"
          aria-label="Toggle navigation menu"
          onClick={() => setOpenAt(open ? null : pathname)}
        >
          {open ? 'Close' : 'Menu'}
        </button>
      </div>
      {open && (
        <nav id="mobile-nav" aria-label="Mobile" className="border-t border-ink-700 px-4 py-3 md:hidden">
          <ul className="flex flex-col gap-1">
            {NAV_LINKS.map((l) => (
              <li key={l.to}>
                <NavLink to={l.to} className={(s) => `block py-2 ${linkClass(s)}`}>
                  {l.label}
                </NavLink>
              </li>
            ))}
            <li>
              <a href={REPO_URL} className="block py-2 text-sm text-slate-300">
                GitHub
              </a>
            </li>
            <li>
              <Link to="/docs/quickstart" className="mt-2 inline-block rounded-md bg-accent px-3 py-2 text-sm font-semibold text-ink-950">
                Get Started
              </Link>
            </li>
          </ul>
        </nav>
      )}
    </header>
  )
}
