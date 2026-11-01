-- A Style's Cover: the one image, from among its design files, that the
-- founder picks to represent the Style (spec #34, ticket #40). The core
-- checks that the file is an image of the same Style. Removing the file
-- clears the Cover.
ALTER TABLE style ADD COLUMN cover_file_id INTEGER REFERENCES style_file (id) ON DELETE SET NULL;
