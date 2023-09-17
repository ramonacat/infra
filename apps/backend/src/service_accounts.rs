use std::sync::Arc;

use rand::{distributions::Alphanumeric, CryptoRng, Rng};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct ServiceAccountToken {
    id: Uuid,
    content: String,
}

impl ServiceAccountToken {
    pub fn create<TCryptoRng: CryptoRng + Rng>(
        id: Uuid,
        csprng: impl (FnOnce() -> TCryptoRng),
    ) -> Self {
        let content = csprng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();

        Self { id, content }
    }
}

#[derive(Debug)]
pub struct ServiceAccount {
    id: Uuid,
    name: String,
    tokens: Vec<ServiceAccountToken>,
}

impl ServiceAccount {
    pub fn create(id: Uuid, name: String) -> Self {
        Self {
            id,
            name,
            tokens: vec![],
        }
    }

    pub fn add_token(&mut self, token: ServiceAccountToken) {
        self.tokens.push(token);
    }
}

pub struct ServiceAccountRepository {
    db_pool: Arc<Pool<Postgres>>,
}

impl ServiceAccountRepository {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self { db_pool }
    }

    pub async fn find_by_name(
        &self,
        name: impl Into<&str> + Send,
    ) -> Result<Option<ServiceAccount>, sqlx::Error> {
        let name: &str = name.into();

        let account = sqlx::query!("SELECT id, name FROM service_accounts WHERE name=$1", name)
            .fetch_optional(self.db_pool.as_ref())
            .await?;

        let Some(account) = account else {
            return Ok(None);
        };

        let tokens = self.find_tokens_for_account(account.id).await?;

        Ok(Some(ServiceAccount {
            id: account.id,
            name: account.name,
            tokens,
        }))
    }

    pub async fn save(&self, account: ServiceAccount) -> Result<(), sqlx::Error> {
        sqlx::query!("INSERT INTO service_accounts (id, name) VALUES($1, $2) ON CONFLICT(id) DO UPDATE SET name = EXCLUDED.name", account.id, account.name)
            .execute(self.db_pool.as_ref())
            .await?;

        for token in account.tokens {
            let current = sqlx::query!(
                "SELECT content FROM service_account_tokens WHERE id=$1",
                token.id
            )
            .fetch_optional(self.db_pool.as_ref())
            .await?;

            if let Some(current) = current {
                if current.content != token.content {
                    sqlx::query!(
                        "UPDATE service_account_tokens SET content = $1 WHERE id = $2",
                        token.content,
                        token.id
                    )
                    .execute(self.db_pool.as_ref())
                    .await?;
                }
            } else {
                sqlx::query!("INSERT INTO service_account_tokens (id, content, service_account) VALUES($1, $2, $3)", token.id, token.content, account.id).execute(self.db_pool.as_ref()).await?;
            }
        }

        Ok(())
    }

    async fn find_tokens_for_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<ServiceAccountToken>, sqlx::Error> {
        sqlx::query_as!(
            ServiceAccountToken,
            "SELECT id, content FROM service_account_tokens WHERE service_account=$1",
            account_id
        )
        .fetch_all(self.db_pool.as_ref())
        .await
    }
}

const ROOT_ACCOUNT_NAME: &str = "root";

pub async fn initialize_root_account<TCryptoRng: CryptoRng + Rng>(
    repository: Arc<ServiceAccountRepository>,
    csprng: impl (FnOnce() -> TCryptoRng) + Send,
) -> Result<(), sqlx::Error> {
    let current_account = repository.find_by_name(ROOT_ACCOUNT_NAME).await?;

    if current_account.is_none() {
        let mut account = ServiceAccount::create(Uuid::new_v4(), ROOT_ACCOUNT_NAME.into());

        account.add_token(ServiceAccountToken::create(Uuid::new_v4(), csprng));

        repository.save(account).await?;
    }

    Ok(())
}
