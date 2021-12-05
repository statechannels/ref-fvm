// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use encoding::{de, from_slice, ser, serde_bytes, to_vec, Cbor, Error as EncodingError};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Method number indicator for calling actor methods.
pub type MethodNum = u64;

/// Base actor send method.
pub const METHOD_SEND: MethodNum = 0;
/// Base actor constructor method.
pub const METHOD_CONSTRUCTOR: MethodNum = 1;
