use rand::{thread_rng, Rng};

pub fn shuffle_vec<T>(mut vec: Vec<T>) -> Vec<T> {
    thread_rng().shuffle(&mut vec);
    vec
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
}
