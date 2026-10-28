import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Link } from 'react-router-dom'
import { CodeBlock } from './CodeBlock'
import { docFileSlugs } from '../lib/docs'
import { REPO_URL } from '../lib/site'

// Documents are written for GitHub, so they link to each other as `file.md`
// and to repository files as `../path`. Map those onto site routes or the repo.
function rewriteDocHref(href: string): string {
  const doc = /^([a-z-]+\.md)(#.*)?$/.exec(href)
  if (doc && docFileSlugs[doc[1]]) return `/docs/${docFileSlugs[doc[1]]}${doc[2] ?? ''}`
  if (href.startsWith('../')) return `${REPO_URL}/blob/main/${href.slice(3)}`
  return href
}

export function Markdown({ source }: { source: string }) {
  return (
    <div className="doc">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a({ href = '', children }) {
            const rewritten = rewriteDocHref(href)
            if (rewritten.startsWith('/')) return <Link to={rewritten}>{children}</Link>
            return (
              <a href={rewritten} target="_blank" rel="noreferrer noopener">
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
