ALTER TABLE preference DROP COLUMN id;
ALTER TABLE preference ADD PRIMARY KEY (chat_id);