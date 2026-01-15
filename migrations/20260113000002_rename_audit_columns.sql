-- Add migration script here
ALTER TABLE ai_audits RENAME COLUMN codigo_generado TO generated_code;
ALTER TABLE ai_audits RENAME COLUMN es_valido TO is_valid;
ALTER TABLE ai_audits RENAME COLUMN error_compilacion TO compilation_error;
