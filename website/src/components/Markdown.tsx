import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Link } from 'react-router-dom'
import { CodeBlock } from './CodeBlock'

export function Markdown({ source }: { source: string }) {
  return (
    <div className="doc">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a({ href = '', children }) {
            if (href.startsWith('/')) return <Link to={href}>{children}</Link>
            return (
              <a href={href} target="_blank" rel="noreferrer noopener">
                {children}
              </a>
            )
          },
          pre({ children }) {
            const child = Array.isArray(children) ? children[0] : children
            const props = (child as { props?: { className?: string; children?: unknown } })?.props
            const lang = /language-(\w+)/.exec(props?.className ?? '')?.[1]
            return <CodeBlock code={String(props?.children ?? '').replace(/\n$/, '')} label={lang} />
          },
        }}
      >
        {source}
      </ReactMarkdown>
    </div>
  )
}
