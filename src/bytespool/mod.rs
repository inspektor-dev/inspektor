// Copyright 2022 poonai
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use bytes::BytesMut;
use std::cell::RefCell;

thread_local!{
    /// BUF_POOL is a thread local storage of pool of BytesMut. It can used to reuse
    /// the allocated memory.
    pub static BUF_POOL: RefCell<BytesMutPool> = RefCell::new(BytesMutPool::default());
}

/// ByteMutPool is used to store BytesMut and it can be retrived when its
/// needed.
#[derive(Default, Debug)]
pub struct BytesMutPool {
    slots: Vec<BytesMut>
}

impl BytesMutPool {
    /// get return BytesMut if it already exist in the slots. if not then it 
    /// allocate a new BytesMut.
    pub fn get(&mut self) -> BytesMut {
        let mut buf = self.slots.pop().unwrap_or(BytesMut::default());
        // reset the BytesMut.
        unsafe {
            buf.set_len(0);
        }
        buf
    }

    /// put will save the BytesMut for later use. if we have already have enough BytesMut
    /// then it's dropped.
    pub fn put(&mut self, buf: BytesMut) {
        if self.slots.len() == 10 {
            return
        }
        self.slots.push(buf)
    }
}