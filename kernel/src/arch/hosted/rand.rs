// SPDX-FileCopyrightText: 2020 Sean Cross <sean@xobs.io>
// SPDX-License-Identifier: Apache-2.0

use core::sync::atomic::Ordering;
use crate::arch::hosted::LOCAL_RNG_STATE;

pub fn get_u32() -> u32 {
    use ::rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    let mut rng = ChaCha8Rng::seed_from_u64(LOCAL_RNG_STATE.load(Ordering::SeqCst));
    let r = rng.next_u32();
    LOCAL_RNG_STATE.store(rng.next_u64(), Ordering::SeqCst);
    r
}
