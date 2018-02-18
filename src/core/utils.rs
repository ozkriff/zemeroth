use rand::{thread_rng, Rng};

pub fn shuffle_vec<T>(mut vec: Vec<T>) -> Vec<T> {
    thread_rng().shuffle(&mut vec);
    vec
}
