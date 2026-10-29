-- The Script of a Documentary (GLOSSARY.md): a document of kind 'script'.
-- Each Documentary has at most one. The core creates it on first open, so
-- older Documentaries get one too (a migration cannot write the file).
ALTER TABLE documentary ADD COLUMN script_id INTEGER REFERENCES document (id);
CREATE UNIQUE INDEX documentary_script ON documentary (script_id);
