// Usage: node validate.mjs <schema.json> <sarif-file>...
// The official SARIF 2.1.0 schema is JSON Schema draft-04, hence ajv-draft-04.
import { readFileSync } from 'node:fs'
import Ajv from 'ajv-draft-04'
import addFormats from 'ajv-formats'

const [schemaPath, ...files] = process.argv.slice(2)
if (!schemaPath || files.length === 0) {
  console.error('usage: node validate.mjs <schema.json> <sarif-file>...')
  process.exit(2)
}

const ajv = new Ajv({ allErrors: true, strict: false })
addFormats(ajv)
const validate = ajv.compile(JSON.parse(readFileSync(schemaPath, 'utf8')))

let failed = false
for (const file of files) {
  const log = JSON.parse(readFileSync(file, 'utf8'))
  if (validate(log)) {
    console.log(`ok    ${file}`)
  } else {
    failed = true
    console.log(`FAIL  ${file}`)
    for (const e of validate.errors.slice(0, 10)) console.log(`      ${e.instancePath || '/'} ${e.message}`)
  }
}
process.exit(failed ? 1 : 0)
