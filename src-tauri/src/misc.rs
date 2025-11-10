use rand::rngs::ThreadRng;
use rand::Rng;

// Weighted randomness. Return index.
pub fn random_with_weights(weights: &[u8], total_option: Option<u8>, rng: &mut ThreadRng) -> usize {
    let total = match total_option {
        Some(t) => t,
        _ => weights.iter().sum()
    };

    let random = rng.random_range(0..total);
    let mut counter = 0;
    for (i, weight) in weights.iter().enumerate() {
        counter += weight;
        if random < counter {
            return i;
        }
    }

    panic!("total weight was {total}, random was {random}");
}