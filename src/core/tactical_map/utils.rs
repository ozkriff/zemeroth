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
}
