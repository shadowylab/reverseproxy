// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use rand::{distributions::Alphanumeric, Rng};

pub fn random_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
