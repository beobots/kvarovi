CREATE INDEX idx_messages_chatid_type ON messages (chat_id, type, created_at DESC);