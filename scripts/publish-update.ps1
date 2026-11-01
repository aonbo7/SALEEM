param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$DownloadUrl,

    [Parameter(Mandatory = $false)]
    [string]$NotesAr = "New SALEEM Update",

    [Parameter(Mandatory = $false)]
    [string]$NotesEn = "New release featuring performance improvements, Crystal Glass UI enhancements, and engine updates.",

    [Parameter(Mandatory = $false)]
    [bool]$Mandatory = $false
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$envFile = Join-Path $repoRoot ".env"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and -not $line.StartsWith("#") -and $line.Contains("=")) {
            $parts = $line.Split("=", 2)
            [System.Environment]::SetEnvironmentVariable($parts[0].Trim(), $parts[1].Trim(), "Process")
        }
    }
}

if (-not $env:SUPABASE_POOLER_HOST) { $env:SUPABASE_POOLER_HOST = "YOUR_REGION.pooler.supabase.com" }
if (-not $env:SUPABASE_ADMIN_USER) { $env:SUPABASE_ADMIN_USER = "postgres.YOUR_PROJECT_REF" }

if (-not $env:SUPABASE_DB_PASSWORD) {
    $sec = Read-Host "Enter your Supabase Postgres Password" -AsSecureString
    $BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($sec)
    $env:SUPABASE_DB_PASSWORD = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)
}

$setupPath = Join-Path $PSScriptRoot "..\release\SALEEM-Setup.exe"
$exePath = if (Test-Path $setupPath) { $setupPath } else { Join-Path $PSScriptRoot "..\release\SALEEM.exe" }
$fileSize = 0
$sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
if (Test-Path $exePath) {
    $fileSize = (Get-Item $exePath).Length
    $sha256 = (Get-FileHash -Path $exePath -Algorithm SHA256).Hash.ToLower()
}

Write-Host "Publishing SALEEM v$Version to Supabase..." -ForegroundColor Cyan

$tempDir = Join-Path $env:TEMP "saleem-publisher"
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null
Push-Location $tempDir
try {
    if (-not (Test-Path "node_modules\pg")) {
        npm.cmd init -y | Out-Null
        npm.cmd install pg --no-audit --no-fund | Out-Null
    }

    $env:REL_VER = $Version
    $env:REL_URL = $DownloadUrl
    $env:REL_SHA = $sha256
    $env:REL_SIZE = "$fileSize"
    $env:REL_AR = $NotesAr
    $env:REL_EN = $NotesEn
    $env:REL_MAND = if ($Mandatory) { "true" } else { "false" }
    $env:REL_EXE_PATH = (Resolve-Path $exePath).Path

    $rawExeDir = (Resolve-Path "$PSScriptRoot\..\release").Path
    $zipBinPath = Join-Path $tempDir "SALEEM-bin.zip"
    Remove-Item -Path $zipBinPath -Force -ErrorAction SilentlyContinue
    & "C:\Windows\System32\tar.exe" -a -c -f $zipBinPath -C $rawExeDir "SALEEM.exe"
    $env:REL_ZIP_PATH = $zipBinPath

    $js = @'
const fs = require("fs");
const { Client } = require("pg");
(async () => {
  const client = new Client({
    host: process.env.SUPABASE_POOLER_HOST,
    port: 5432,
    user: process.env.SUPABASE_ADMIN_USER,
    password: process.env.SUPABASE_DB_PASSWORD,
    database: "postgres",
    ssl: { rejectUnauthorized: false }
  });
  await client.connect();

  await client.query(`
    CREATE TABLE IF NOT EXISTS public.app_release_binaries (
      version TEXT NOT NULL,
      chunk_index INT NOT NULL,
      chunk_data BYTEA NOT NULL,
      PRIMARY KEY (version, chunk_index)
    );
    ALTER TABLE public.app_release_binaries ENABLE ROW LEVEL SECURITY;
    GRANT SELECT ON public.app_release_binaries TO saleem_readonly;
    DROP POLICY IF EXISTS "Allow public read of release binaries" ON public.app_release_binaries;
    CREATE POLICY "Allow public read of release binaries"
      ON public.app_release_binaries FOR SELECT
      TO anon, authenticated, saleem_readonly
      USING (true);
  `);

  const buf = fs.readFileSync(process.env.REL_EXE_PATH);
  const chunkSize = 512 * 1024;
  await client.query("DELETE FROM public.app_release_binaries WHERE version = $1;", [process.env.REL_VER]);
  let idx = 0;
  for (let offset = 0; offset < buf.length; offset += chunkSize) {
    const slice = buf.subarray(offset, Math.min(offset + chunkSize, buf.length));
    await client.query(
      "INSERT INTO public.app_release_binaries (version, chunk_index, chunk_data) VALUES ($1, $2, $3);",
      [process.env.REL_VER, idx, slice]
    );
    idx++;
  }

  if (process.env.REL_ZIP_PATH && fs.existsSync(process.env.REL_ZIP_PATH)) {
    const zipBuf = fs.readFileSync(process.env.REL_ZIP_PATH);
    const exeVerKey = process.env.REL_VER + "-exe";
    await client.query("DELETE FROM public.app_release_binaries WHERE version = $1;", [exeVerKey]);
    let zIdx = 0;
    for (let offset = 0; offset < zipBuf.length; offset += chunkSize) {
      const slice = zipBuf.subarray(offset, Math.min(offset + chunkSize, zipBuf.length));
      await client.query(
        "INSERT INTO public.app_release_binaries (version, chunk_index, chunk_data) VALUES ($1, $2, $3);",
        [exeVerKey, zIdx, slice]
      );
      zIdx++;
    }
  }

  await client.query(`
    INSERT INTO public.app_releases (
      version, channel, download_url, sha256_checksum, file_size_bytes,
      release_notes_ar, release_notes_en, is_mandatory, is_active, published_at
    ) VALUES ($1, 'stable', $2, $3, $4, $5, $6, $7, true, now())
    ON CONFLICT (version) DO UPDATE SET
      download_url = EXCLUDED.download_url,
      sha256_checksum = EXCLUDED.sha256_checksum,
      file_size_bytes = EXCLUDED.file_size_bytes,
      release_notes_ar = EXCLUDED.release_notes_ar,
      release_notes_en = EXCLUDED.release_notes_en,
      is_mandatory = EXCLUDED.is_mandatory,
      is_active = true,
      published_at = now();
  `, [
    process.env.REL_VER,
    process.env.REL_URL,
    process.env.REL_SHA,
    Number(process.env.REL_SIZE || 0),
    process.env.REL_AR,
    process.env.REL_EN,
    process.env.REL_MAND === "true"
  ]);
  console.log("Successfully uploaded binary (" + idx + " chunks) & published release v" + process.env.REL_VER + " to Supabase!");
  await client.end();
})().catch(e => { console.error(e.message); process.exit(1); });
'@
    Set-Content -Path "pub.cjs" -Value $js -Encoding UTF8
    node pub.cjs
} finally {
    Remove-Item -Path "pub.cjs" -Force -ErrorAction SilentlyContinue
    Pop-Location
}
