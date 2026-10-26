import type { ReactNode } from 'react'

export function Page({ title, lead, children }: { title: string; lead?: string; children: ReactNode }) {
  return (
    <div className="mx-auto max-w-4xl px-4 py-12">
      <h1 className="text-3xl font-bold text-white md:text-4xl">{title}</h1>
      {lead && <p className="mt-3 text-lg text-slate-300">{lead}</p>}
      <div className="mt-8 space-y-6 text-slate-300">{children}</div>
    </div>
  )
}

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h2 className="mb-2 text-xl font-semibold text-white">{title}</h2>
      <div className="space-y-3 leading-7">{children}</div>
    </section>
  )
}
