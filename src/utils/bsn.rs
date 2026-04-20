use rand::RngExt;
use secrecy::SecretString;

/// Generate a random but valid BSN (Burgerservicenummer) as a SecretString.
///
/// A valid BSN is 9 digits and satisfies the 11-proof check:
/// weights 9,8,7,6,5,4,3,2,-1 applied to digits left-to-right,
/// with the weighted sum divisible by 11.
pub fn random_bsn() -> SecretString {
    let mut rng = rand::rng();
    loop {
        // Use 999 prefix so generated BSNs fall in the designated test range
        let prefix: Vec<u32> = vec![9, 9, 9];
        let random_part: Vec<u32> = (0..5).map(|_| rng.random_range(0..10)).collect();
        let digits: Vec<u32> = prefix.into_iter().chain(random_part).collect();
        let weights = [9, 8, 7, 6, 5, 4, 3, 2];
        let partial_sum: i32 = digits
            .iter()
            .zip(weights.iter())
            .map(|(&d, &w)| d as i32 * w)
            .sum();

        // last digit weight is -1, so: (partial_sum - last_digit) % 11 == 0
        let remainder = partial_sum.rem_euclid(11);
        if remainder > 9 {
            continue;
        }
        let last_digit = remainder as u32;

        let bsn: String = digits
            .iter()
            .chain(std::iter::once(&last_digit))
            .map(|d| char::from_digit(*d, 10).unwrap())
            .collect();

        // reject all-zeros
        if bsn == "000000000" {
            continue;
        }

        return SecretString::from(bsn);
    }
}
