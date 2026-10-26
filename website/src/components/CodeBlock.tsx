import { useState } from 'react'

export function CodeBlock({ code, label, wrap = false }: { code: string; label?: string; wrap?: boolean }) {
  const [copied, setCopied] = useState(false)
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      setCopied(false)
    }
  }
  return (
    <figure className="my-4 overflow-hidden rounded-lg border border-ink-700 bg-ink-900">
      <figcaption className="flex items-center justify-between border-b border-ink-700 px-3 py-1.5 text-xs text-slate-400">
        <span>{label ?? 'code'}</span>
        <button type="button" onClick={copy} className="rounded px-2 py-0.5 hover:bg-ink-700 hover:text-white">
          {copied ? 'Copied' : 'Copy'}
        </button>
      </figcaption>
      <pre tabIndex={0} className={`overflow-x-auto p-4 font-mono text-sm leading-6 text-slate-200 ${wrap ? 'whitespace-pre-wrap break-words' : ''}`}>
        <code>{code}</code>
      </pre>
    </figure>
  )
}
