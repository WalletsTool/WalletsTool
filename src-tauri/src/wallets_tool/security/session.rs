use lazy_static::lazy_static;
use rand::Rng;
use std::sync::Mutex;
use zeroize::Zeroize;

lazy_static! {
    static ref SESSION_KEY: Mutex<[u8; 32]> = Mutex::new({
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    });
}

pub fn get_session_key() -> [u8; 32] {
    let guard = SESSION_KEY.lock().unwrap();
    *guard
}

#[allow(dead_code)]
pub fn clear_session_key() {
    let mut guard = SESSION_KEY.lock().unwrap();
    guard.zeroize();
}
