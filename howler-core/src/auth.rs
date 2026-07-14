use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Researcher,
    Viewer,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::Researcher => "Researcher",
            UserRole::Viewer => "Viewer",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Admin" => Ok(UserRole::Admin),
            "Researcher" => Ok(UserRole::Researcher),
            "Viewer" => Ok(UserRole::Viewer),
            _ => anyhow::bail!("Invalid role: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub struct AuthService<'a> {
    conn: &'a Connection,
}

impl<'a> AuthService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        AuthService { conn }
    }

    pub fn init_auth_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    username TEXT NOT NULL UNIQUE,
                    email TEXT NOT NULL UNIQUE,
                    password_hash TEXT NOT NULL,
                    role TEXT NOT NULL DEFAULT 'Viewer',
                    created_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS sessions (
                    user_id TEXT NOT NULL,
                    token TEXT PRIMARY KEY,
                    expires_at TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(id)
                );

                CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
                CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);",
            )
            .context("Failed to create auth schema")?;
        Ok(())
    }

    pub fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
        role: UserRole,
    ) -> Result<User> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        self.conn
            .execute(
                "INSERT INTO users (id, username, email, password_hash, role, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    username,
                    email,
                    password_hash,
                    role.as_str(),
                    created_at.to_rfc3339(),
                ],
            )
            .context("Failed to insert user")?;

        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            role,
            created_at,
        })
    }

    pub fn login(&self, username: &str, password: &str) -> Result<Session> {
        let user = self
            .get_user_by_username(username)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("Invalid password"))?;

        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let expires_at = Utc::now() + chrono::Duration::hours(24);

        self.conn
            .execute("DELETE FROM sessions WHERE user_id = ?1", params![user.id])
            .context("Failed to clean old sessions")?;

        self.conn
            .execute(
                "INSERT INTO sessions (user_id, token, expires_at)
                 VALUES (?1, ?2, ?3)",
                params![user.id, token, expires_at.to_rfc3339()],
            )
            .context("Failed to create session")?;

        Ok(Session {
            user_id: user.id,
            token,
            expires_at,
        })
    }

    pub fn validate_token(&self, token: &str) -> Result<Option<User>> {
        let result = self.conn.query_row(
            "SELECT u.id, u.username, u.email, u.password_hash, u.role, u.created_at
             FROM users u
             JOIN sessions s ON u.id = s.user_id
             WHERE s.token = ?1 AND s.expires_at > datetime('now')",
            params![token],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: UserRole::from_str(&row.get::<_, String>(4)?).unwrap_or(UserRole::Viewer),
                    created_at: {
                        let s: String = row.get(5)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                })
            },
        );

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let result = self.conn.query_row(
            "SELECT id, username, email, password_hash, role, created_at
             FROM users WHERE id = ?1",
            params![user_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: UserRole::from_str(&row.get::<_, String>(4)?).unwrap_or(UserRole::Viewer),
                    created_at: {
                        let s: String = row.get(5)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                })
            },
        );

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let result = self.conn.query_row(
            "SELECT id, username, email, password_hash, role, created_at
             FROM users WHERE username = ?1",
            params![username],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: UserRole::from_str(&row.get::<_, String>(4)?).unwrap_or(UserRole::Viewer),
                    created_at: {
                        let s: String = row.get(5)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                })
            },
        );

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn logout(&self, token: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM sessions WHERE token = ?1", params![token])
            .context("Failed to delete session")?;
        Ok(())
    }

    pub fn list_users(&self) -> Result<Vec<User>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, username, email, password_hash, role, created_at FROM users")?;

        let users = stmt
            .query_map([], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: UserRole::from_str(&row.get::<_, String>(4)?).unwrap_or(UserRole::Viewer),
                    created_at: {
                        let s: String = row.get(5)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }
}
