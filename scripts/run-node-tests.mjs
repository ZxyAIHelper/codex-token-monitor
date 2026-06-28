import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { join } from "node:path";

const testDir = join(".test-dist", "tests");
const testFiles = readdirSync(testDir)
  .filter((file) => file.endsWith(".js"))
  .map((file) => join(testDir, file));

if (testFiles.length === 0) {
  console.error(`No compiled test files found in ${testDir}`);
  process.exit(1);
}

const result = spawnSync(process.execPath, ["--test", ...testFiles], {
  stdio: "inherit",
});

process.exit(result.status ?? 1);
