mod toon;
pub mod converter;

// WASM bindings (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use converter::{json_to_toon as json_to_toon_internal, toon_to_json as toon_to_json_internal};

// UniFFI bindings (not for WASM)
#[cfg(not(target_arch = "wasm32"))]
mod uniffi_bindings {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    /// Error type for TOON conversion operations
    #[derive(Debug, thiserror::Error, uniffi::Error)]
    pub enum ToonError {
        #[error("Conversion error: {message}")]
        ConversionError { message: String },
    }

    impl From<String> for ToonError {
        fn from(message: String) -> Self {
            ToonError::ConversionError { message }
        }
    }

    /// Convert JSON string to TOON format (stateless)
    #[uniffi::export]
    pub fn json_to_toon(json_data: String) -> Result<String, ToonError> {
        json_to_toon_internal(&json_data).map_err(ToonError::from)
    }

    /// Convert TOON format to JSON string (stateless)
    #[uniffi::export]
    pub fn toon_to_json(toon_data: String) -> Result<String, ToonError> {
        toon_to_json_internal(&toon_data).map_err(ToonError::from)
    }

    /// Cached converter with Moka (in-memory) + optional Sled (persistent) storage
    #[derive(uniffi::Object)]
    pub struct CachedConverter {
        #[cfg(feature = "cache")]
        moka_cache: Option<Arc<moka::sync::Cache<String, String>>>,
        #[cfg(feature = "persistent-cache")]
        sled_db: Option<Arc<Mutex<sled::Db>>>,
    }

    #[uniffi::export]
    impl CachedConverter {
        /// Create a new cached converter
        /// 
        /// # Arguments
        /// * `cache_size` - Maximum number of entries in memory cache (0 = disabled)
        /// * `cache_ttl_secs` - Time-to-live in seconds (None = no expiration)
        /// * `persistent_path` - Optional path for persistent Sled cache
        #[uniffi::constructor]
        pub fn new(cache_size: u64, cache_ttl_secs: Option<u64>, persistent_path: Option<String>) -> Arc<Self> {
            #[cfg(feature = "cache")]
            let moka_cache = if cache_size > 0 {
                let mut builder = moka::sync::Cache::builder().max_capacity(cache_size);
                if let Some(ttl) = cache_ttl_secs {
                    builder = builder.time_to_live(std::time::Duration::from_secs(ttl));
                }
                Some(Arc::new(builder.build()))
            } else {
                None
            };
            #[cfg(not(feature = "cache"))]
            let moka_cache = None;

            #[cfg(feature = "persistent-cache")]
            let sled_db = persistent_path.and_then(|path| {
                sled::open(&path).ok().map(|db| Arc::new(Mutex::new(db)))
            });
            #[cfg(not(feature = "persistent-cache"))]
            let sled_db = None;

            Arc::new(Self {
                #[cfg(feature = "cache")]
                moka_cache,
                #[cfg(feature = "persistent-cache")]
                sled_db,
            })
        }

        /// Convert JSON to TOON with caching
        pub fn json_to_toon(&self, json_data: String) -> Result<String, ToonError> {
            let cache_key = format!("j2t:{}", json_data);
            
            // Check Moka cache (hot path)
            #[cfg(feature = "cache")]
            if let Some(ref cache) = self.moka_cache {
                if let Some(cached) = cache.get(&cache_key) {
                    return Ok(cached);
                }
            }

            // Check Sled cache (cold path)
            #[cfg(feature = "persistent-cache")]
            if let Some(ref db) = self.sled_db {
                if let Ok(db_lock) = db.lock() {
                    if let Ok(Some(cached)) = db_lock.get(cache_key.as_bytes()) {
                        if let Ok(result) = String::from_utf8(cached.to_vec()) {
                            // Warm up Moka cache
                            #[cfg(feature = "cache")]
                            if let Some(ref cache) = self.moka_cache {
                                cache.insert(cache_key.clone(), result.clone());
                            }
                            return Ok(result);
                        }
                    }
                }
            }

            // Cache miss - perform conversion
            let result = json_to_toon_internal(&json_data).map_err(ToonError::from)?;

            // Store in both caches
            #[cfg(feature = "cache")]
            if let Some(ref cache) = self.moka_cache {
                cache.insert(cache_key.clone(), result.clone());
            }

            #[cfg(feature = "persistent-cache")]
            if let Some(ref db) = self.sled_db {
                if let Ok(db_lock) = db.lock() {
                    let _ = db_lock.insert(cache_key.as_bytes(), result.as_bytes());
                    let _ = db_lock.flush();
                }
            }

            Ok(result)
        }

        /// Convert TOON to JSON with caching
        pub fn toon_to_json(&self, toon_data: String) -> Result<String, ToonError> {
            let cache_key = format!("t2j:{}", toon_data);
            
            // Check Moka cache (hot path)
            #[cfg(feature = "cache")]
            if let Some(ref cache) = self.moka_cache {
                if let Some(cached) = cache.get(&cache_key) {
                    return Ok(cached);
                }
            }

            // Check Sled cache (cold path)
            #[cfg(feature = "persistent-cache")]
            if let Some(ref db) = self.sled_db {
                if let Ok(db_lock) = db.lock() {
                    if let Ok(Some(cached)) = db_lock.get(cache_key.as_bytes()) {
                        if let Ok(result) = String::from_utf8(cached.to_vec()) {
                            // Warm up Moka cache
                            #[cfg(feature = "cache")]
                            if let Some(ref cache) = self.moka_cache {
                                cache.insert(cache_key.clone(), result.clone());
                            }
                            return Ok(result);
                        }
                    }
                }
            }

            // Cache miss - perform conversion
            let result = toon_to_json_internal(&toon_data).map_err(ToonError::from)?;

            // Store in both caches
            #[cfg(feature = "cache")]
            if let Some(ref cache) = self.moka_cache {
                cache.insert(cache_key.clone(), result.clone());
            }

            #[cfg(feature = "persistent-cache")]
            if let Some(ref db) = self.sled_db {
                if let Ok(db_lock) = db.lock() {
                    let _ = db_lock.insert(cache_key.as_bytes(), result.as_bytes());
                    let _ = db_lock.flush();
                }
            }

            Ok(result)
        }

        /// Clear all caches
        pub fn clear_cache(&self) {
            #[cfg(feature = "cache")]
            if let Some(ref cache) = self.moka_cache {
                cache.invalidate_all();
            }

            #[cfg(feature = "persistent-cache")]
            if let Some(ref db) = self.sled_db {
                if let Ok(db_lock) = db.lock() {
                    let _ = db_lock.clear();
                    let _ = db_lock.flush();
                }
            }
        }

        /// Get cache statistics
        pub fn cache_stats(&self) -> String {
            let mut stats = String::from("Cache Statistics:\n");
            
            #[cfg(feature = "cache")]
            if let Some(ref cache) = self.moka_cache {
                // Sync cache state to get accurate counts
                cache.run_pending_tasks();
                stats.push_str(&format!("  Moka entries: {}\n", cache.entry_count()));
                stats.push_str(&format!("  Moka weighted size: {} bytes\n", cache.weighted_size()));
            } else {
                stats.push_str("  Moka: disabled\n");
            }

            #[cfg(not(feature = "cache"))]
            {
                stats.push_str("  Moka: disabled\n");
            }

            #[cfg(feature = "persistent-cache")]
            if let Some(ref db) = self.sled_db {
                if let Ok(db_lock) = db.lock() {
                    stats.push_str(&format!("  Sled entries: {}\n", db_lock.len()));
                } else {
                    stats.push_str("  Sled: locked\n");
                }
            } else {
                stats.push_str("  Sled: disabled\n");
            }

            #[cfg(not(feature = "persistent-cache"))]
            {
                stats.push_str("  Sled: disabled\n");
            }

            stats
        }
    }
}

// Re-export for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub use uniffi_bindings::*;

// Generate the UniFFI scaffolding (0.29+ uses proc-macros) - only for non-WASM
#[cfg(not(target_arch = "wasm32"))]
uniffi::setup_scaffolding!();

