use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TleData {
    pub line1: String,
    pub line2: String,
    pub name: String,
}

impl TleData {
    pub fn to_elements(&self) -> Result<sgp4::Elements, String> {
        sgp4::Elements::from_tle(
            None,
            self.line1.as_bytes(),
            self.line2.as_bytes(),
        ).map_err(|e| format!("TLE parsing error: {:?}", e))
    }
}

#[derive(Serialize, Deserialize)]
struct TleCache {
    data: HashMap<String, TleData>,
    downloaded_at: i64, // Unix timestamp
}

pub struct TleLoader {
    cache_dir: String,
    cache_file: String,
    cache_max_age_hours: u64,
}

impl TleLoader {
    pub fn new() -> Self {
        let cache_dir = "cache".to_string();
        let cache_file = format!("{}/tle_cache.json", cache_dir);
        
        Self {
            cache_dir,
            cache_file,
            cache_max_age_hours: 24, // Cache for 24 hours
        }
    }

    /// Get cache directory path
    fn cache_path(&self) -> &Path {
        Path::new(&self.cache_dir)
    }

    /// Get cache file path
    fn cache_file_path(&self) -> &Path {
        Path::new(&self.cache_file)
    }

    /// Check if cache exists and is still valid
    fn is_cache_valid(&self) -> bool {
        let cache_path = self.cache_file_path();
        
        if !cache_path.exists() {
            return false;
        }

        // Check file modification time
        if let Ok(metadata) = fs::metadata(cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let cache_age_hours = duration.as_secs() / 3600;
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() / 3600;
                    
                    let age = current_time.saturating_sub(cache_age_hours);
                    return age < self.cache_max_age_hours;
                }
            }
        }
        
        false
    }

    /// Load TLE data from cache
    fn load_from_cache(&self) -> Result<HashMap<String, TleData>, Box<dyn std::error::Error>> {
        let cache_path = self.cache_file_path();
        
        if !cache_path.exists() {
            return Err("Cache file does not exist".into());
        }

        let contents = fs::read_to_string(cache_path)?;
        let cache: TleCache = serde_json::from_str(&contents)?;
        
        println!("✓ Loaded {} satellites from cache (downloaded at {})", 
            cache.data.len(),
            DateTime::<Utc>::from_timestamp(cache.downloaded_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "unknown".to_string()));
        
        Ok(cache.data)
    }

    /// Save TLE data to cache
    fn save_to_cache(&self, data: &HashMap<String, TleData>) -> Result<(), Box<dyn std::error::Error>> {
        // Create cache directory if it doesn't exist
        if let Some(parent) = self.cache_path().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(self.cache_path())?;

        let cache = TleCache {
            data: data.clone(),
            downloaded_at: Utc::now().timestamp(),
        };

        let json = serde_json::to_string_pretty(&cache)?;
        fs::write(self.cache_file_path(), json)?;
        
        println!("✓ Cached {} satellites to {}", data.len(), self.cache_file);
        
        Ok(())
    }

    /// Download TLE data from Celestrak
    fn download_tle_data(&self) -> Result<HashMap<String, TleData>, Box<dyn std::error::Error>> {
        println!("Downloading TLE data from Celestrak...");
        
        // Download active satellites TLE data
        let url = "https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle";
        let response = reqwest::blocking::get(url)?;
        let text = response.text()?;

        let mut satellites = HashMap::new();
        let lines: Vec<&str> = text.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            if lines[i].trim().is_empty() {
                i += 1;
                continue;
            }

            let name = lines[i].trim().to_string();
            
            if i + 2 < lines.len() {
                let line1 = lines[i + 1].trim().to_string();
                let line2 = lines[i + 2].trim().to_string();
                
                // Validate TLE format (line1 should start with "1 ", line2 with "2 ")
                if line1.starts_with("1 ") && line2.starts_with("2 ") {
                    satellites.insert(
                        name.clone(),
                        TleData {
                            name,
                            line1,
                            line2,
                        },
                    );
                }
            }
            
            i += 3;
        }

        println!("✓ Downloaded {} satellites from Celestrak", satellites.len());
        
        // Save to cache
        if let Err(e) = self.save_to_cache(&satellites) {
            eprintln!("Warning: Failed to save cache: {}", e);
        }

        Ok(satellites)
    }

    /// Load active satellites (with caching)
    pub fn load_active_satellites(&self) -> Result<HashMap<String, TleData>, Box<dyn std::error::Error>> {
        // Check if cache is valid
        if self.is_cache_valid() {
            match self.load_from_cache() {
                Ok(data) => {
                    println!("Using cached TLE data (cache is less than {} hours old)", 
                        self.cache_max_age_hours);
                    return Ok(data);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load cache: {}. Downloading fresh data...", e);
                }
            }
        } else {
            if self.cache_file_path().exists() {
                println!("Cache is expired (older than {} hours). Downloading fresh data...", 
                    self.cache_max_age_hours);
            } else {
                println!("No cache found. Downloading TLE data...");
            }
        }

        // Download fresh data
        self.download_tle_data()
    }

    /// Clear the cache (useful for testing or forcing refresh)
    pub fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = self.cache_file_path();
        if cache_path.exists() {
            fs::remove_file(cache_path)?;
            println!("Cache cleared");
        }
        Ok(())
    }
}

