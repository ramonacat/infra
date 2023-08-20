CREATE TABLE service_accounts (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE service_account_tokens (
    id UUID PRIMARY KEY,
    service_account UUID REFERENCES service_accounts(id) ON DELETE CASCADE,
    content TEXT NOT NULL
);