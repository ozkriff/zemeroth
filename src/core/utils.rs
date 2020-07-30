use std::fmt::Debug;

use quad_rand::compat::QuadRand;
use rand::{
    distributions::uniform::{SampleBorrow, SampleUniform},
    seq::SliceRandom,
    Rng,
};

pub fn zrng() -> impl rand::Rng {
    QuadRand
}

pub fn roll_dice<T: SampleUniform, B1, B2>(low: B1, high: B2) -> T
where
    B1: SampleBorrow<T> + Sized,
    B2: SampleBorrow<T> + Sized,
{
    zrng().gen_range(low, high)
}

pub fn shuffle_vec<T>(mut vec: Vec<T>) -> Vec<T> {
    vec.shuffle(&mut zrng());
    vec
}

/// Remove an element from a vector.
pub fn try_remove_item<T: Debug + PartialEq>(vec: &mut Vec<T>, e: &T) -> bool {
    vec.iter()
        .position(|current| current == e)
        .map(|e| vec.remove(e))
        .is_some()
}

pub fn clamp_min<T: PartialOrd>(value: T, min: T) -> T {
    if value < min {
        min
    } else {
        value
    }
}

pub fn clamp_max<T: PartialOrd>(value: T, max: T) -> T {
    if value > max {
        max
    } else {
        value
    }
}

pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clamp_min() {
        assert_eq!(super::clamp_min(1, 0), 1);
        assert_eq!(super::clamp_min(0, 0), 0);
        assert_eq!(super::clamp_min(-1, 0), 0);
    }

    #[test]
    fn test_clamp_max() {
        assert_eq!(super::clamp_max(1, 2), 1);
        assert_eq!(super::clamp_max(2, 2), 2);
        assert_eq!(super::clamp_max(3, 2), 2);
    }

    #[test]
    fn test_clamp() {
        let min = 0;
        let max = 2;
        assert_eq!(super::clamp(1, min, max), 1);
        assert_eq!(super::clamp(0, min, max), 0);
        assert_eq!(super::clamp(-1, min, max), 0);
        assert_eq!(super::clamp(1, min, max), 1);
        assert_eq!(super::clamp(2, min, max), 2);
        assert_eq!(super::clamp(3, min, max), 2);
    }

    #[test]
    fn test_try_remove_item() {
        let mut a = vec![1, 2, 3];
        assert!(super::try_remove_item(&mut a, &1));
        assert_eq!(&a, &[2, 3]);
        assert!(!super::try_remove_item(&mut a, &666));
    }
}
