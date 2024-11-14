use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserPreview {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub name: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct User {
    id: Option<Uuid>, // Added ID field to store after insertion
    name: String,
    username: String,
    #[cfg_attr(feature = "hydrate", allow(dead_code))]
    #[serde(skip_serializing)]
    password: Option<String>,
    email: String,
    email_hash: String, // Added email_hash field
    bio: Option<String>,
    image: Option<String>,
}

#[cfg(feature = "ssr")]
static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

impl User {
    #[inline]
    pub fn id(&self) -> Option<uuid::Uuid> {
        self.id
    }

    #[inline]
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    #[inline]
    pub fn username(&self) -> String {
        self.username.to_string()
    }

    #[inline]
    pub fn email(&self) -> String {
        self.email.to_string()
    }

    #[inline]
    pub fn bio(&self) -> Option<String> {
        self.bio.clone()
    }

    #[inline]
    pub fn image(&self) -> Option<String> {
        self.image.clone()
    }

    pub fn set_password(mut self, password: String) -> Result<Self, String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".into());
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase || !has_lowercase || !has_digit || !has_special {
            return Err(
                "Password must contain uppercase, lowercase, number, and special character".into(),
            );
        }

        self.password = Some(password);
        Ok(self)
    }

    pub fn set_name(mut self, name: String) -> Result<Self, String> {
        if name.len() < 4 {
            return Err(format!("Name {name} is too short, at least 4 characters"));
        }
        self.name = name;
        Ok(self)
    }

    // pub fn set_username(mut self, username: String) -> Result<Self, String> {
    //     if username.len() < 4 {
    //         return Err(format!(
    //             "Username {username} is too short, at least 4 characters"
    //         ));
    //     }
    //     self.username = username;
    //     Ok(self)
    // }

    #[cfg(feature = "ssr")]
    fn validate_email(email: &str) -> bool {
        EMAIL_REGEX
            .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
            .is_match(email)
    }

    #[cfg(not(feature = "ssr"))]
    fn validate_email(email: &str) -> bool {
        crate::emailRegex(email)
    }

    pub fn set_email(&mut self, email: String) -> Result<&mut Self, String> {
        if !Self::validate_email(&email) {
            return Err(format!(
                "The email {email} is invalid, provide a correct one"
            ));
        }
        self.email = email.clone();
        self.email_hash = Self::hash_email(&email); // Generate the email hash
        Ok(self)
    }

    #[warn(dead_code)]
    pub fn set_bio(&mut self, bio: String) -> Result<&mut Self, String> {
        static BIO_MIN: usize = 10;
        if bio.is_empty() {
            self.bio = None;
        } else if bio.len() < BIO_MIN {
            return Err("bio too short, at least 10 characters".into());
        } else {
            self.bio = Some(bio);
        }
        Ok(self)
    }

    #[warn(dead_code)]
    pub fn set_image(&mut self, image: String) -> Result<&mut Self, String> {
        if image.is_empty() {
            self.image = None;
        } else if !image.starts_with("http") {
            return Err("Invalid image!".into());
        } else {
            self.image = Some(image);
        }
        Ok(self)
    }

    fn hash_email(email: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(email);
        format!("{:x}", hasher.finalize()) // Returns the hash as a hex string
    }

    #[warn(dead_code)]
    fn generate_username(name: &str) -> String {
        let words: Vec<&str> = name.split_whitespace().collect();
        let random_string: String = std::iter::repeat(())
            .map(|()| rand::thread_rng().sample(Alphanumeric))
            .take(7)
            .map(char::from)
            .collect();

        let username = match words.as_slice() {
            [first, last, ..] => format!(
                "{}_{}_{}",
                first.to_lowercase(),
                last.to_lowercase(),
                random_string
            ),
            [first] => format!("{}_{}", first.to_lowercase(), random_string),
            _ => random_string.clone(),
        };

        // Ensure username meets length requirements
        if username.len() < 4 {
            format!("user_{}", random_string)
        } else if username.len() > 30 {
            format!("{}_{}", &username[..20], random_string)
        } else {
            username
        }
    }

    #[cfg(feature = "ssr")]
    pub async fn get(user_id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, username, email, email_hash, bio, image, NULL as password FROM users WHERE id=$1",
            user_id
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_by_id(user_id: uuid::Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, email, null as password, email_hash, username, bio, image FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn insert(&mut self) -> Result<Uuid, sqlx::Error> {
        let pool = crate::database::get_db();
        let mut tx = pool.begin().await?;

        if self.username.is_empty() {
            self.username = User::generate_username(&self.name);
        }

        let result = sqlx::query!(
            "INSERT INTO Users(name, username, email, email_hash, password) 
             VALUES ($1, $2, $3, $4, crypt($5, gen_salt('bf'))) 
             RETURNING id",
            self.name,
            self.username,
            self.email,
            self.email_hash,
            self.password,
        )
        .fetch_one(&mut *tx) // Note the &mut *tx here
        .await;

        match result {
            Ok(record) => {
                tx.commit().await?;
                self.id = Some(record.id);
                Ok(record.id)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    #[cfg(feature = "ssr")]
    pub async fn update(&self) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "
            UPDATE Users SET
                image=$2,
                bio=$3,
                email=$4,
                email_hash=$5,
                password=CASE WHEN $6 THEN crypt($7, gen_salt('bf')) ELSE password END
            WHERE id=$1",
            self.id,
            self.image,
            self.bio,
            self.email,
            self.email_hash,
            self.password.is_some(),
            self.password,
        )
        .execute(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    #[allow(dead_code)]
    pub async fn update_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE Users 
         SET password = crypt($3, gen_salt('bf'))
         WHERE id = $1 
         AND password = crypt($2, password)
         RETURNING id",
            self.id,
            old_password,
            new_password
        )
        .fetch_optional(crate::database::get_db())
        .await?;

        Ok(result.is_some())
    }
}
