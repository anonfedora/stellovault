// Lightweight test runner for dashboard helper functions.
// Uses jiti to execute the TypeScript sources directly without a build step,
// and node:assert for invariants. Invoked via `npm run test:dashboard`.
import { createJiti } from "jiti";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const jiti = createJiti(import.meta.url, {
  alias: { "@": path.join(root, "src") },
  interopDefault: true,
  jsx: { runtime: "automatic" },
});

const testFiles = [
  "src/utils/dashboardExport.test.ts",
  "src/hooks/useDashboard.aggregation.test.ts",
];

let passed = 0;
let failed = 0;
const failures = [];

for (const rel of testFiles) {
  const target = path.join(root, rel);
  process.stdout.write(`\nRunning ${rel}\n`);
  try {
    const mod = await jiti.import(target);
    const tests = mod.tests ?? mod.default;
    if (!Array.isArray(tests)) {
      throw new Error(`${rel} must export an array named "tests"`);
    }
    for (const { name, run } of tests) {
      try {
        await run();
        passed += 1;
        process.stdout.write(`  ✓ ${name}\n`);
      } catch (err) {
        failed += 1;
        failures.push({ file: rel, name, err });
        process.stdout.write(`  ✗ ${name}\n`);
      }
    }
  } catch (err) {
    failed += 1;
    failures.push({ file: rel, name: "<import>", err });
    process.stdout.write(`  ✗ failed to load: ${err.message}\n`);
  }
}

process.stdout.write(`\n${passed} passed, ${failed} failed\n`);
if (failed > 0) {
  for (const { file, name, err } of failures) {
    process.stderr.write(`\n[${file}] ${name}\n${err.stack ?? err}\n`);
  }
  process.exit(1);
}
