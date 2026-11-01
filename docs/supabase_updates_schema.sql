-- ============================================================================
-- SALEEM — Supabase Cloud Updates & Live Rules Schema
-- Run this script once in your Supabase Dashboard -> SQL Editor -> New Query
-- ============================================================================

-- 1. Table for Application Releases (Auto-Updater)
CREATE TABLE IF NOT EXISTS public.app_releases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version TEXT NOT NULL UNIQUE,                  -- e.g. '0.2.0'
    channel TEXT NOT NULL DEFAULT 'stable',        -- 'stable' or 'beta'
    download_url TEXT NOT NULL,                    -- Direct HTTPS link to SALEEM-Setup.exe (Supabase Storage or GitHub Release)
    sha256_checksum TEXT NOT NULL,                 -- 64-char hex SHA-256 of the installer for integrity verification
    file_size_bytes BIGINT NOT NULL DEFAULT 0,     -- Installer size in bytes
    release_notes_ar TEXT NOT NULL DEFAULT '',     -- Arabic changelog shown in the glass update modal
    release_notes_en TEXT NOT NULL DEFAULT '',     -- English changelog shown in the glass update modal
    is_mandatory BOOLEAN NOT NULL DEFAULT false,   -- If true, prompts user immediately on startup
    is_active BOOLEAN NOT NULL DEFAULT true,       -- Allows unpublishing a broken release instantly
    published_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Table for Live Engine Rules (Cleanup, Leftovers & Sentinel Security Rules)
CREATE TABLE IF NOT EXISTS public.remote_rules_catalog (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    engine_scope TEXT NOT NULL UNIQUE,             -- 'cleanup', 'sentinel', or 'inventory_games'
    rules_version TEXT NOT NULL,                   -- e.g. '2026.11.01'
    payload_json JSONB NOT NULL,                   -- Declarative rules JSON array
    sha256_checksum TEXT NOT NULL,                 -- SHA-256 of payload_json
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. Enable Row Level Security (RLS) so users can ONLY read, never modify!
ALTER TABLE public.app_releases ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.remote_rules_catalog ENABLE ROW LEVEL SECURITY;

-- Allow read-only (SELECT) access for desktop clients using the anon public key
DROP POLICY IF EXISTS "Allow public read of active releases" ON public.app_releases;
CREATE POLICY "Allow public read of active releases"
    ON public.app_releases
    FOR SELECT
    TO anon, authenticated
    USING (is_active = true);

DROP POLICY IF EXISTS "Allow public read of remote rules" ON public.remote_rules_catalog;
CREATE POLICY "Allow public read of remote rules"
    ON public.remote_rules_catalog
    FOR SELECT
    TO anon, authenticated
    USING (true);

-- 4. Seed initial v0.1.0 release row as an example
INSERT INTO public.app_releases (
    version,
    channel,
    download_url,
    sha256_checksum,
    file_size_bytes,
    release_notes_ar,
    release_notes_en,
    is_mandatory,
    is_active
) VALUES (
    '0.1.0',
    'stable',
    'https://github.com/saleem-project/SALEEM/releases/download/v0.1.0/SALEEM-Setup.exe',
    '0000000000000000000000000000000000000000000000000000000000000000',
    8847360,
    'الإصدار الأولي: محلل الأقراص، مكتشف التكرارات، التنظيف الآمن، وجرد البرامج والألعاب بواجهة Crystal Glass.',
    'Initial release: Disk Analyzer, Duplicate Finder, Safe Cleanup, and App/Game Leftover Inventory with Crystal Glass UI.',
    false,
    true
) ON CONFLICT (version) DO NOTHING;
