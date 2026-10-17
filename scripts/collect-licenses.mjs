import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";

const execFileAsync = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const licenseFilePattern = /^(license|licence|copying|notice|unlicense)([-_. ].*)?$/i;

async function readLicenseText(packageDirectory, explicitLicenseFile) {
  if (explicitLicenseFile) {
    const explicitPath = resolve(packageDirectory, explicitLicenseFile);
    try {
      return await readFile(explicitPath, "utf8");
    } catch (error) {
      if (error.code !== "ENOENT") {
        throw error;
      }
    }
  }

  const fileNames = (await readdir(packageDirectory, { withFileTypes: true }))
    .filter((entry) => entry.isFile() && licenseFilePattern.test(entry.name))
    .map((entry) => entry.name)
    .sort();
  const texts = await Promise.all(
    fileNames.map(async (fileName) => ({
      fileName,
      text: await readFile(join(packageDirectory, fileName), "utf8"),
    })),
  );

  return texts
    .map(({ fileName, text }) => `===== ${fileName} =====\n${text.trim()}`)
    .join("\n\n");
}

function addEntry(entries, entry) {
  const key = `${entry.ecosystem}:${entry.name}@${entry.version}:${entry.license}`;
  const existing = entries.get(key);
  if (!existing || (!existing.licenseText && entry.licenseText)) {
    entries.set(key, entry);
  }
}

async function collectNpmLicenses(entries) {
  const lock = JSON.parse(await readFile(join(root, "package-lock.json"), "utf8"));

  for (const [packagePath, lockedPackage] of Object.entries(lock.packages ?? {})) {
    if (!packagePath.startsWith("node_modules/") || lockedPackage.dev) {
      continue;
    }

    const packageDirectory = join(root, packagePath);
    let packageJson;
    try {
      packageJson = JSON.parse(
        await readFile(join(packageDirectory, "package.json"), "utf8"),
      );
    } catch (error) {
      if (lockedPackage.optional && error.code === "ENOENT") {
        addEntry(entries, {
          ecosystem: "npm",
          name: lockedPackage.name ?? packagePath.slice("node_modules/".length),
          version: lockedPackage.version ?? "不明",
          license: normalizeLicense(lockedPackage.license),
          licenseText: "",
        });
        continue;
      }
      throw error;
    }
    const license = normalizeLicense(lockedPackage.license ?? packageJson.license);
    addEntry(entries, {
      ecosystem: "npm",
      name: packageJson.name ?? packagePath.slice("node_modules/".length),
      version: lockedPackage.version ?? packageJson.version ?? "不明",
      license,
      licenseText: await readLicenseText(
        packageDirectory,
        packageJson.licenseFile,
      ),
    });
  }
}

async function collectCargoLicenses(entries) {
  const projectLicenseText = await readFile(join(root, "LICENSE"), "utf8");
  const manifests = [
    {
      path: "src-rs/src-tauri/Cargo.toml",
      workspace: false,
    },
    {
      path: "src-rs/src-plugin/anydex-plugin-api/Cargo.toml",
      workspace: false,
    },
    {
      path: "src-rs/src-plugin/anydex-plugin-default-impl/Cargo.toml",
      workspace: true,
    },
  ];

  for (const manifest of manifests) {
    const { stdout } = await execFileAsync(
      "cargo",
      [
        "about",
        "generate",
        "--format",
        "json",
        "--fail",
        "--all-features",
        "--locked",
        "--config",
        join(root, "about.toml"),
        ...(manifest.workspace ? ["--workspace"] : []),
        "--manifest-path",
        join(root, manifest.path),
      ],
      { cwd: root, maxBuffer: 128 * 1024 * 1024 },
    );
    const report = JSON.parse(stdout);
    const licenseTextsByPackage = new Map();
    for (const license of report.licenses ?? []) {
      for (const { crate } of license.used_by ?? []) {
        const key = `${crate.name}@${crate.version}`;
        const texts = licenseTextsByPackage.get(key) ?? [];
        const text = `===== ${license.name} (${license.id}) =====\n${license.text.trim()}`;
        if (!texts.includes(text)) {
          texts.push(text);
          licenseTextsByPackage.set(key, texts);
        }
      }
    }

    for (const dependency of report.crates ?? []) {
      const packageKey = `${dependency.package.name}@${dependency.package.version}`;
      addEntry(entries, {
        ecosystem: "Cargo",
        name: dependency.package.name,
        version: dependency.package.version,
        license: dependency.package.license ?? dependency.license,
        licenseText:
          dependency.package.source == null
            ? projectLicenseText
            : (licenseTextsByPackage.get(packageKey) ?? []).join("\n\n"),
      });
    }
  }
}

function normalizeLicense(license) {
  if (typeof license === "string") {
    return license;
  }
  if (license && typeof license === "object" && "type" in license) {
    return license.type;
  }
  return "ライセンス情報なし";
}

async function main() {
  const entries = new Map();
  await collectNpmLicenses(entries);
  await collectCargoLicenses(entries);

  const licenses = [...entries.values()].sort(
    (left, right) =>
      left.ecosystem.localeCompare(right.ecosystem) ||
      left.name.localeCompare(right.name) ||
      left.version.localeCompare(right.version),
  );
  const output = join(root, "public", "licenses.json");
  await mkdir(dirname(output), { recursive: true });
  await writeFile(output, `${JSON.stringify(licenses)}\n`);
  console.log(`Collected ${licenses.length} dependency licenses in ${output}`);
}

main().catch((error) => {
  console.error("Failed to collect dependency licenses:", error);
  if (error.stderr) {
    console.error(error.stderr);
  }
  process.exitCode = 1;
});
