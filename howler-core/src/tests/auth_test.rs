use crate::auth::{AuthService, UserRole};
use crate::annotations::{AnnotationStore, AnnotationType};
use chrono::Utc;
use rusqlite::Connection;
use tempfile::NamedTempFile;

fn setup_db() -> (NamedTempFile, Connection) {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();

    // Create sightings table first (needed for FK references)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sightings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            species TEXT NOT NULL,
            scientific_name TEXT,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            observed_on TEXT NOT NULL,
            source TEXT NOT NULL,
            source_id TEXT NOT NULL,
            details TEXT,
            UNIQUE(source, source_id)
        );",
    )
    .unwrap();

    // Insert a test sighting so FK references work
    conn.execute(
        "INSERT INTO sightings (species, scientific_name, latitude, longitude, observed_on, source, source_id, details)
         VALUES ('Canis lupus', 'Canis lupus', 45.0, -122.0, ?1, 'GBIF', 'test_1', 'Test sighting')",
        rusqlite::params![Utc::now().to_rfc3339()],
    )
    .unwrap();

    (temp_file, conn)
}

#[test]
fn test_user_registration() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Researcher)
        .unwrap();

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.role, UserRole::Researcher);
    assert!(!user.id.is_empty());
}

#[test]
fn test_user_login_success() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    auth.register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let session = auth.login("testuser", "password123").unwrap();
    assert!(!session.token.is_empty());
    assert!(!session.user_id.is_empty());
}

#[test]
fn test_user_login_wrong_password() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    auth.register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let result = auth.login("testuser", "wrongpassword");
    assert!(result.is_err());
}

#[test]
fn test_user_login_nonexistent_user() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let result = auth.login("nonexistent", "password123");
    assert!(result.is_err());
}

#[test]
fn test_session_token_validation() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    auth.register("testuser", "test@example.com", "password123", UserRole::Admin)
        .unwrap();

    let session = auth.login("testuser", "password123").unwrap();

    let user = auth.validate_token(&session.token).unwrap();
    assert!(user.is_some());
    let user = user.unwrap();
    assert_eq!(user.username, "testuser");
    assert_eq!(user.role, UserRole::Admin);
}

#[test]
fn test_session_token_invalid() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let user = auth.validate_token("invalid_token").unwrap();
    assert!(user.is_none());
}

#[test]
fn test_get_user() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let fetched = auth.get_user(&user.id).unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().username, "testuser");
}

#[test]
fn test_duplicate_username() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    auth.register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let result = auth.register("testuser", "other@example.com", "password456", UserRole::Viewer);
    assert!(result.is_err());
}

#[test]
fn test_create_annotation() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Researcher)
        .unwrap();

    let annotation = store
        .create_annotation(1, &user.id, "This is a test comment", AnnotationType::Comment)
        .unwrap();

    assert_eq!(annotation.sighting_id, 1);
    assert_eq!(annotation.user_id, user.id);
    assert_eq!(annotation.text, "This is a test comment");
    assert_eq!(annotation.annotation_type, AnnotationType::Comment);
}

#[test]
fn test_get_annotations_for_sighting() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    store
        .create_annotation(1, &user.id, "Comment 1", AnnotationType::Comment)
        .unwrap();
    store
        .create_annotation(1, &user.id, "Note 1", AnnotationType::Note)
        .unwrap();
    store
        .create_annotation(1, &user.id, "Comment 2", AnnotationType::Comment)
        .unwrap();

    let annotations = store.get_annotations_for_sighting(1).unwrap();
    assert_eq!(annotations.len(), 3);

    let annotations_2 = store.get_annotations_for_sighting(2).unwrap();
    assert!(annotations_2.is_empty());
}

#[test]
fn test_update_annotation() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let annotation = store
        .create_annotation(1, &user.id, "Original text", AnnotationType::Comment)
        .unwrap();

    store.update_annotation(&annotation.id, "Updated text").unwrap();

    let annotations = store.get_annotations_for_sighting(1).unwrap();
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].text, "Updated text");
}

#[test]
fn test_delete_annotation() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let annotation = store
        .create_annotation(1, &user.id, "To be deleted", AnnotationType::Comment)
        .unwrap();

    store.delete_annotation(&annotation.id).unwrap();

    let annotations = store.get_annotations_for_sighting(1).unwrap();
    assert!(annotations.is_empty());
}

#[test]
fn test_add_and_get_rating() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Researcher)
        .unwrap();

    let rating = store
        .add_rating(1, &user.id, 4, Some("Looks like a gray wolf"))
        .unwrap();

    assert_eq!(rating.sighting_id, 1);
    assert_eq!(rating.confidence, 4);
    assert_eq!(rating.notes, Some("Looks like a gray wolf".to_string()));

    let ratings = store.get_ratings_for_sighting(1).unwrap();
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0].confidence, 4);
}

#[test]
fn test_average_rating() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user1 = auth
        .register("user1", "user1@example.com", "pass1", UserRole::Viewer)
        .unwrap();
    let user2 = auth
        .register("user2", "user2@example.com", "pass2", UserRole::Viewer)
        .unwrap();

    store.add_rating(1, &user1.id, 3, None).unwrap();
    store.add_rating(1, &user2.id, 5, None).unwrap();

    let avg = store.get_average_rating(1).unwrap();
    assert!(avg.is_some());
    let avg = avg.unwrap();
    assert!((avg - 4.0).abs() < 0.01);
}

#[test]
fn test_invalid_rating_confidence() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    let store = AnnotationStore::new(&conn);
    store.init_annotation_schema().unwrap();

    let user = auth
        .register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let result = store.add_rating(1, &user.id, 6, None);
    assert!(result.is_err());

    let result = store.add_rating(1, &user.id, 0, None);
    assert!(result.is_err());
}

#[test]
fn test_logout() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    auth.register("testuser", "test@example.com", "password123", UserRole::Viewer)
        .unwrap();

    let session = auth.login("testuser", "password123").unwrap();
    assert!(auth.validate_token(&session.token).unwrap().is_some());

    auth.logout(&session.token).unwrap();
    assert!(auth.validate_token(&session.token).unwrap().is_none());
}

#[test]
fn test_list_users() {
    let (_temp, conn) = setup_db();
    let auth = AuthService::new(&conn);
    auth.init_auth_schema().unwrap();

    auth.register("user1", "u1@example.com", "pass1", UserRole::Admin)
        .unwrap();
    auth.register("user2", "u2@example.com", "pass2", UserRole::Viewer)
        .unwrap();

    let users = auth.list_users().unwrap();
    assert_eq!(users.len(), 2);
}
