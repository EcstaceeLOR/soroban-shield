import { Link } from 'react-router-dom'

export default function NotFound() {
  return (
    <div className="mx-auto max-w-3xl px-4 py-20 text-center">
      <p className="font-mono text-accent">404</p>
      <h1 className="mt-2 text-3xl font-bold text-white">Page not found</h1>
      <p className="mt-3 text-slate-300">The page you are looking for does not exist.</p>
      <div className="mt-6 flex justify-center gap-4 text-accent underline">
        <Link to="/">Home</Link>
        <Link to="/docs">Documentation</Link>
        <Link to="/rules">Security rules</Link>
      </div>
    </div>
  )
}
