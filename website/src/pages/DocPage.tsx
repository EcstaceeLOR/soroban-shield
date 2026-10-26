import { Markdown } from '../components/Markdown'
import type { DocPage as Doc } from '../lib/docs'

export default function DocPage({ doc }: { doc: Doc }) {
  return <Markdown source={doc.body} />
}
