import { Route, Routes } from 'react-router-dom'
import { MarketingLayout } from '../layouts/MarketingLayout'
import { DocsLayout } from '../layouts/DocsLayout'
import { docPages } from '../lib/docs'
import Home from '../pages/Home'
import Features from '../pages/Features'
import HowItWorks from '../pages/HowItWorks'
import Rules from '../pages/Rules'
import RuleDetail from '../pages/RuleDetail'
import DocsHome from '../pages/DocsHome'
import DocPage from '../pages/DocPage'
import Contributing from '../pages/Contributing'
import Roadmap from '../pages/Roadmap'
import About from '../pages/About'
import NotFound from '../pages/NotFound'

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<MarketingLayout />}>
        <Route index element={<Home />} />
        <Route path="features" element={<Features />} />
        <Route path="how-it-works" element={<HowItWorks />} />
        <Route path="rules" element={<Rules />} />
        <Route path="rules/:ruleId" element={<RuleDetail />} />
        <Route path="docs" element={<DocsLayout />}>
          <Route index element={<DocsHome />} />
          {docPages.map((d) => (
            <Route key={d.slug} path={d.slug} element={<DocPage doc={d} />} />
          ))}
        </Route>
        <Route path="contributing" element={<Contributing />} />
        <Route path="roadmap" element={<Roadmap />} />
        <Route path="about" element={<About />} />
        <Route path="*" element={<NotFound />} />
      </Route>
    </Routes>
  )
}
