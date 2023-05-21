use rand::Rng;

pub fn reservoir_sample<T, I: Iterator<Item = T>, R: Rng>(
    rng: &mut R,
    mut source: I,
    k: usize,
) -> Vec<T> {
    let mut samples = Vec::new();

    for _ in 0..k {
        if let Some(x) = source.next() {
            samples.push(x);
        } else {
            break;
        }
    }

    let mut i = k + 1;
    for sample in source {
        let j = rng.gen_range(0..i);
        if j < k {
            samples[j] = sample;
        }
        i += 1;
    }

    samples
}
