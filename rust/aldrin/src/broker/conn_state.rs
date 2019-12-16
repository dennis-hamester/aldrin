// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::BrokerEvent;
use futures_channel::mpsc::Sender;
use futures_util::sink::SinkExt;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct ConnectionState {
    send: Sender<BrokerEvent>,
    objects: HashSet<Uuid>,
    objects_created_subscribed: bool,
}

impl ConnectionState {
    pub fn new(send: Sender<BrokerEvent>) -> Self {
        ConnectionState {
            send,
            objects: HashSet::new(),
            objects_created_subscribed: false,
        }
    }

    pub fn add_object(&mut self, id: Uuid) {
        let unique = self.objects.insert(id);
        debug_assert!(unique);
    }

    pub fn remove_object(&mut self, id: Uuid) {
        let contained = self.objects.remove(&id);
        debug_assert!(contained);
    }

    pub fn objects<'a>(&'a self) -> impl Iterator<Item = Uuid> + 'a {
        self.objects.iter().cloned()
    }

    pub async fn send(&mut self, ev: BrokerEvent) -> Result<(), ()> {
        self.send.send(ev).await.map_err(|_| ())
    }

    pub fn set_objects_created_subscribed(&mut self, subscribed: bool) {
        self.objects_created_subscribed = subscribed;
    }

    pub fn objects_created_subscribed(&self) -> bool {
        self.objects_created_subscribed
    }
}
