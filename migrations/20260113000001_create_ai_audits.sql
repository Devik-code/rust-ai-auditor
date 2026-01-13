-- Create ai_audits table
CREATE TABLE IF NOT EXISTS ai_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prompt TEXT NOT NULL,
    codigo_generado TEXT NOT NULL,
    es_valido BOOLEAN NOT NULL,
    error_compilacion TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on created_at for faster queries
CREATE INDEX idx_ai_audits_created_at ON ai_audits(created_at DESC);
