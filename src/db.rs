use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use chrono::{Utc, Datelike};
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};

const MAX_PLAYS_PER_PLAYER: i32 = 3;

/// Database manager for Tamagotogether.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Initializes a new database connection and ensures tables exist.
    pub fn new(path: &str) -> Result<Arc<Self>> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {}", path))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS actions (
                ip TEXT NOT NULL,
                action TEXT NOT NULL,
                date TEXT NOT NULL
            )",
            [],
        ).context("Failed to create actions table")?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_actions_date ON actions (date)",
            [],
        ).context("Failed to create index on actions table")?;
        
        Ok(Arc::new(Db { conn: Mutex::new(conn) }))
    }

    /// Returns the current date as a string in YYYY-MM-DD format.
    fn today_str() -> String {
        let now = Utc::now();
        format!("{}-{:02}-{:02}", now.year(), now.month(), now.day())
    }

    /// Hashes an IP address using SHA-256.
    fn hash_ip(ip: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generates an initial mood for a given date.
    pub fn get_initial_mood(&self, date: &str) -> i32 {
        let mut hasher = Sha256::new();
        hasher.update(date.as_bytes());
        let result = hasher.finalize();
        (result[0] as i32 % 5) + 1
    }

    /// Calculates the current happiness level.
    pub fn get_level(&self) -> Result<i32> {
        let today = Self::today_str();
        let initial = self.get_initial_mood(&today);
        let count = self.get_feed_count_today()?;
        Ok(std::cmp::min(initial + count, 10))
    }

    /// Checks if a given IP has already fed the Tamagotchi today.
    pub fn has_fed_today(&self, ip: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        let today = Self::today_str();
        let hashed_ip = Self::hash_ip(ip);
        let mut stmt = conn.prepare("SELECT 1 FROM actions WHERE ip = ?1 AND date = ?2 AND action = 'feed'")?;
        let exists = stmt.exists(params![hashed_ip, today])?;
        Ok(exists)
    }

    /// Feeds the Tamagotchi.
    pub fn feed(&self, ip: &str) -> Result<i32> {
        let today = Self::today_str();
        let hashed_ip = Self::hash_ip(ip);
        
        if self.has_fed_today(ip)? {
            return self.get_level();
        }
        
        {
            let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
            conn.execute(
                "INSERT INTO actions (ip, action, date) VALUES (?1, 'feed', ?2)",
                params![hashed_ip, today],
            ).context("Failed to insert feed action")?;
        }
        
        self.get_level()
    }

    /// Returns the total number of times the Tamagotchi was fed today.
    pub fn get_feed_count_today(&self) -> Result<i32> {
        let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        let today = Self::today_str();
        conn.query_row(
            "SELECT COUNT(*) FROM actions WHERE date = ?1 AND action = 'feed'",
            params![today],
            |row| row.get(0),
        ).context("Failed to query feed count")
    }

    // --- Play feature ---

    /// Returns how many times a specific player played today.
    pub fn get_player_play_count_today(&self, ip: &str) -> Result<i32> {
        let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        let today = Self::today_str();
        let hashed_ip = Self::hash_ip(ip);
        conn.query_row(
            "SELECT COUNT(*) FROM actions WHERE ip = ?1 AND date = ?2 AND action = 'play'",
            params![hashed_ip, today],
            |row| row.get(0),
        ).context("Failed to query player play count")
    }

    /// Returns the total number of play actions today.
    pub fn get_play_count_today(&self) -> Result<i32> {
        let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        let today = Self::today_str();
        conn.query_row(
            "SELECT COUNT(*) FROM actions WHERE date = ?1 AND action = 'play'",
            params![today],
            |row| row.get(0),
        ).context("Failed to query play count")
    }

    /// Returns the current playfulness level (1-10), based on total plays today.
    /// Increases by 1 every 3 plays, starting at level 1.
    pub fn get_playfulness_level(&self) -> Result<i32> {
        let count = self.get_play_count_today()?;
        Ok(std::cmp::min(1 + count / 3, 10))
    }

    /// Registers a play action for the given IP. Max 3 per player per day.
    /// Returns the new playfulness level.
    pub fn play(&self, ip: &str) -> Result<i32> {
        let player_count = self.get_player_play_count_today(ip)?;
        if player_count >= MAX_PLAYS_PER_PLAYER {
            return self.get_playfulness_level();
        }

        let today = Self::today_str();
        let hashed_ip = Self::hash_ip(ip);
        {
            let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
            conn.execute(
                "INSERT INTO actions (ip, action, date) VALUES (?1, 'play', ?2)",
                params![hashed_ip, today],
            ).context("Failed to insert play action")?;
        }

        self.get_playfulness_level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_initialization() {
        let db = Db::new(":memory:").expect("Failed to create in-memory db");
        assert_eq!(db.get_feed_count_today().unwrap(), 0);
        assert_eq!(db.get_play_count_today().unwrap(), 0);
    }

    #[test]
    fn test_feed() {
        let db = Db::new(":memory:").unwrap();
        let initial_level = db.get_level().unwrap();
        
        let level1 = db.feed("192.168.1.1").unwrap();
        assert_eq!(db.get_feed_count_today().unwrap(), 1);
        assert_eq!(db.has_fed_today("192.168.1.1").unwrap(), true);
        assert_eq!(level1, std::cmp::min(initial_level + 1, 10));
        
        // Feeding again from same IP should not increase count
        let level2 = db.feed("192.168.1.1").unwrap();
        assert_eq!(db.get_feed_count_today().unwrap(), 1);
        assert_eq!(level1, level2);
    }

    #[test]
    fn test_play() {
        let db = Db::new(":memory:").unwrap();
        let ip = "10.0.0.1";
        
        // Initial playfulness level: 1 + 0/3 = 1
        assert_eq!(db.get_playfulness_level().unwrap(), 1);

        // 1st play
        let p_level1 = db.play(ip).unwrap();
        assert_eq!(db.get_play_count_today().unwrap(), 1);
        assert_eq!(db.get_player_play_count_today(ip).unwrap(), 1);
        assert_eq!(p_level1, 1);

        // 2nd play
        db.play(ip).unwrap();
        
        // 3rd play
        let p_level3 = db.play(ip).unwrap();
        assert_eq!(db.get_player_play_count_today(ip).unwrap(), 3);
        assert_eq!(db.get_play_count_today().unwrap(), 3);
        assert_eq!(p_level3, 2); // 1 + 3/3 = 2

        // 4th play ignored
        let p_level4 = db.play(ip).unwrap();
        assert_eq!(db.get_player_play_count_today(ip).unwrap(), 3);
        assert_eq!(db.get_play_count_today().unwrap(), 3);
        assert_eq!(p_level4, 2);
    }
}
