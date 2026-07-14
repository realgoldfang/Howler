use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationType {
    Comment,
    Classification,
    Note,
}

impl AnnotationType {
    pub fn as_str(&self) -> &str {
        match self {
            AnnotationType::Comment => "Comment",
            AnnotationType::Classification => "Classification",
            AnnotationType::Note => "Note",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Comment" => Ok(AnnotationType::Comment),
            "Classification" => Ok(AnnotationType::Classification),
            "Note" => Ok(AnnotationType::Note),
            _ => anyhow::bail!("Invalid annotation type: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub id: String,
    pub sighting_id: i64,
    pub user_id: String,
    pub text: String,
    pub annotation_type: AnnotationType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SightingRating {
    pub id: String,
    pub sighting_id: i64,
    pub user_id: String,
    pub confidence: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct AnnotationStore<'a> {
    conn: &'a Connection,
}

impl<'a> AnnotationStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        AnnotationStore { conn }
    }

    pub fn init_annotation_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS annotations (
                    id TEXT PRIMARY KEY,
                    sighting_id INTEGER NOT NULL,
                    user_id TEXT NOT NULL,
                    text TEXT NOT NULL,
                    annotation_type TEXT NOT NULL DEFAULT 'Comment',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (sighting_id) REFERENCES sightings(id),
                    FOREIGN KEY (user_id) REFERENCES users(id)
                );

                CREATE TABLE IF NOT EXISTS sighting_ratings (
                    id TEXT PRIMARY KEY,
                    sighting_id INTEGER NOT NULL,
                    user_id TEXT NOT NULL,
                    confidence INTEGER NOT NULL CHECK(confidence >= 1 AND confidence <= 5),
                    notes TEXT,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (sighting_id) REFERENCES sightings(id),
                    FOREIGN KEY (user_id) REFERENCES users(id),
                    UNIQUE(sighting_id, user_id)
                );

                CREATE INDEX IF NOT EXISTS idx_annotations_sighting ON annotations(sighting_id);
                CREATE INDEX IF NOT EXISTS idx_annotations_user ON annotations(user_id);
                CREATE INDEX IF NOT EXISTS idx_ratings_sighting ON sighting_ratings(sighting_id);",
            )
            .context("Failed to create annotation schema")?;
        Ok(())
    }

    pub fn create_annotation(
        &self,
        sighting_id: i64,
        user_id: &str,
        text: &str,
        annotation_type: AnnotationType,
    ) -> Result<Annotation> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        self.conn
            .execute(
                "INSERT INTO annotations (id, sighting_id, user_id, text, annotation_type, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    sighting_id,
                    user_id,
                    text,
                    annotation_type.as_str(),
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )
            .context("Failed to insert annotation")?;

        Ok(Annotation {
            id,
            sighting_id,
            user_id: user_id.to_string(),
            text: text.to_string(),
            annotation_type,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_annotations_for_sighting(&self, sighting_id: i64) -> Result<Vec<Annotation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sighting_id, user_id, text, annotation_type, created_at, updated_at
             FROM annotations WHERE sighting_id = ?1 ORDER BY created_at ASC",
        )?;

        let annotations = stmt
            .query_map(params![sighting_id], |row| {
                Ok(Annotation {
                    id: row.get(0)?,
                    sighting_id: row.get(1)?,
                    user_id: row.get(2)?,
                    text: row.get(3)?,
                    annotation_type: AnnotationType::from_str(&row.get::<_, String>(4)?)
                        .unwrap_or(AnnotationType::Comment),
                    created_at: {
                        let s: String = row.get(5)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                    updated_at: {
                        let s: String = row.get(6)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(annotations)
    }

    pub fn update_annotation(&self, annotation_id: &str, text: &str) -> Result<()> {
        let now = Utc::now();
        let affected = self.conn.execute(
            "UPDATE annotations SET text = ?1, updated_at = ?2 WHERE id = ?3",
            params![text, now.to_rfc3339(), annotation_id],
        )?;

        if affected == 0 {
            anyhow::bail!("Annotation not found");
        }
        Ok(())
    }

    pub fn delete_annotation(&self, annotation_id: &str) -> Result<()> {
        let affected = self.conn.execute(
            "DELETE FROM annotations WHERE id = ?1",
            params![annotation_id],
        )?;

        if affected == 0 {
            anyhow::bail!("Annotation not found");
        }
        Ok(())
    }

    pub fn add_rating(
        &self,
        sighting_id: i64,
        user_id: &str,
        confidence: i32,
        notes: Option<&str>,
    ) -> Result<SightingRating> {
        if !(1..=5).contains(&confidence) {
            anyhow::bail!("Confidence must be between 1 and 5");
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO sighting_ratings (id, sighting_id, user_id, confidence, notes, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    sighting_id,
                    user_id,
                    confidence,
                    notes,
                    now.to_rfc3339(),
                ],
            )
            .context("Failed to insert rating")?;

        Ok(SightingRating {
            id,
            sighting_id,
            user_id: user_id.to_string(),
            confidence,
            notes: notes.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub fn get_ratings_for_sighting(&self, sighting_id: i64) -> Result<Vec<SightingRating>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, sighting_id, user_id, confidence, notes, created_at
             FROM sighting_ratings WHERE sighting_id = ?1",
        )?;

        let ratings = stmt
            .query_map(params![sighting_id], |row| {
                Ok(SightingRating {
                    id: row.get(0)?,
                    sighting_id: row.get(1)?,
                    user_id: row.get(2)?,
                    confidence: row.get(3)?,
                    notes: row.get(4)?,
                    created_at: {
                        let s: String = row.get(5)?;
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ratings)
    }

    pub fn get_average_rating(&self, sighting_id: i64) -> Result<Option<f64>> {
        let result = self.conn.query_row(
            "SELECT AVG(confidence) FROM sighting_ratings WHERE sighting_id = ?1",
            params![sighting_id],
            |row| row.get::<_, Option<f64>>(0),
        )?;

        Ok(result)
    }
}
