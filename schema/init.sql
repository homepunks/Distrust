CREATE TABLE IF NOT EXISTS pastes (
	id TEXT PRIMARY KEY,
    	content BLOB NOT NULL,
    	content_type TEXT NOT NULL DEFAULT 'text/plain',
	size INTEGER NOT NULL,
	created_at INTEGER NOT NULL,
	view_count INTEGER DEFAULT 0
);
