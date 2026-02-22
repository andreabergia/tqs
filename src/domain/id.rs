use rand::Rng;

static ADJECTIVES: std::sync::OnceLock<Vec<&str>> = std::sync::OnceLock::new();
static NOUNS: std::sync::OnceLock<Vec<&str>> = std::sync::OnceLock::new();

fn init_adjectives() -> &'static Vec<&'static str> {
    ADJECTIVES.get_or_init(|| include_str!("../data/adjectives.txt").lines().collect())
}

fn init_nouns() -> &'static Vec<&'static str> {
    NOUNS.get_or_init(|| include_str!("../data/nouns.txt").lines().collect())
}

pub fn adjectives() -> &'static [&'static str] {
    init_adjectives().as_slice()
}

pub fn nouns() -> &'static [&'static str] {
    init_nouns().as_slice()
}

const MAX_ATTEMPTS: u32 = 100;

pub struct IdGenerator<F>
where
    F: Fn(&str) -> bool,
{
    exists_fn: F,
}

impl<F> IdGenerator<F>
where
    F: Fn(&str) -> bool,
{
    pub fn new(exists_fn: F) -> Self {
        Self { exists_fn }
    }

    pub fn generate(&self) -> String {
        let mut rng = rand::thread_rng();

        for _ in 0..MAX_ATTEMPTS {
            let adjective = adjectives()[rng.gen_range(0..adjectives().len())];
            let noun = nouns()[rng.gen_range(0..nouns().len())];
            let suffix: u16 = rng.gen_range(0..u16::MAX);
            let id = format!("{adjective}-{noun}-{suffix:04x}");

            if !(self.exists_fn)(&id) {
                return id;
            }
        }

        panic!("Failed to generate unique ID after {MAX_ATTEMPTS} attempts")
    }
}

#[cfg(test)]
mod tests {
    use super::{IdGenerator, adjectives, nouns};

    #[test]
    fn wordlists_have_reasonable_size() {
        assert!(adjectives().len() >= 256, "adjectives list too small");
        assert!(nouns().len() >= 256, "nouns list too small");
    }

    #[test]
    fn id_follows_word_word_hex_format() {
        let generator = IdGenerator::new(|_| false);
        let id = generator.generate();

        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(
            parts.len(),
            3,
            "ID should have 3 parts separated by hyphens"
        );

        let hex_suffix = u16::from_str_radix(parts[2], 16);
        assert!(hex_suffix.is_ok(), "suffix should be valid hex");
    }

    #[test]
    fn generate_avoids_collisions() {
        let mut used_ids = std::collections::HashSet::new();

        for _ in 0..100 {
            let existing_ids = used_ids.clone();
            let generator = IdGenerator::new(move |id| existing_ids.contains(id));

            let id = generator.generate();
            assert!(!used_ids.contains(&id), "generated ID should be unique");
            used_ids.insert(id);
        }
    }

    #[test]
    fn generate_retries_on_collision() {
        let collision_count = std::sync::atomic::AtomicU32::new(0);

        let generator = IdGenerator::new(|id| {
            if id == "test-word-1234" {
                collision_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                true
            } else {
                false
            }
        });

        let id = generator.generate();
        assert_ne!(id, "test-word-1234");
    }

    #[test]
    #[should_panic(expected = "Failed to generate unique ID after 100 attempts")]
    fn generate_panics_after_max_attempts() {
        let generator = IdGenerator::new(|_| true);
        let _ = generator.generate();
    }
}
