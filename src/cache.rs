use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub struct Cache<V> {
    entries: HashMap<String, Entry<V>>,
    pending: HashSet<String>,
    capacity: usize,
    lifetime: Duration,
    clock: u64,
}

struct Entry<V> {
    value: V,
    used: u64,
    stored: Instant,
}

impl<V: Clone> Cache<V> {
    pub fn new(capacity: usize, lifetime: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            pending: HashSet::new(),
            capacity,
            lifetime,
            clock: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<V> {
        self.expire(key);
        self.clock += 1;
        let entry = self.entries.get_mut(key)?;
        entry.used = self.clock;
        Some(entry.value.clone())
    }

    pub fn claim(&mut self, key: &str) -> bool {
        self.expire(key);
        !self.entries.contains_key(key) && self.pending.insert(key.to_owned())
    }

    fn expire(&mut self, key: &str) {
        if self
            .entries
            .get(key)
            .is_some_and(|entry| entry.stored.elapsed() >= self.lifetime)
        {
            self.entries.remove(key);
        }
    }

    pub fn fail(&mut self, key: &str) {
        self.pending.remove(key);
    }

    pub fn insert(&mut self, key: String, value: V) {
        self.pending.remove(&key);
        self.clock += 1;
        self.entries.insert(
            key,
            Entry {
                value,
                used: self.clock,
                stored: Instant::now(),
            },
        );
        while self.entries.len() > self.capacity {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.used)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.entries.remove(&oldest);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cache;
    use std::time::Duration;

    const FOREVER: Duration = Duration::MAX;

    #[test]
    fn claims_once_until_settled() {
        let mut cache = Cache::new(2, FOREVER);
        assert!(cache.claim("a"));
        assert!(!cache.claim("a"));
        cache.fail("a");
        assert!(cache.claim("a"));
        cache.insert("a".into(), 1);
        assert!(!cache.claim("a"));
    }

    #[test]
    fn evicts_least_recently_used() {
        let mut cache = Cache::new(2, FOREVER);
        cache.insert("a".into(), 1);
        cache.insert("b".into(), 2);
        assert_eq!(cache.get("a"), Some(1));
        cache.insert("c".into(), 3);
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("a"), Some(1));
        assert_eq!(cache.get("c"), Some(3));
    }

    #[test]
    fn expires_stale_entries() {
        let mut cache = Cache::new(2, Duration::ZERO);
        cache.insert("a".into(), 1);
        assert_eq!(cache.get("a"), None);
        assert!(cache.claim("a"));
    }
}
