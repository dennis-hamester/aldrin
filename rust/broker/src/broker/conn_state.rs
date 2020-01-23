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

use super::{BrokerEvent, ObjectCookie, ServiceCookie};
use futures_channel::mpsc::Sender;
use futures_util::sink::SinkExt;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct ConnectionState {
    send: Sender<BrokerEvent>,
    objects: HashSet<ObjectCookie>,
    objects_created_subscribed: bool,
    objects_destroyed_subscribed: bool,
    services_created_subscribed: bool,
    services_destroyed_subscribed: bool,

    /// Map of active subscriptions made by this connection.
    subscriptions: HashMap<ServiceCookie, HashSet<u32>>,
}

impl ConnectionState {
    pub fn new(send: Sender<BrokerEvent>) -> Self {
        ConnectionState {
            send,
            objects: HashSet::new(),
            objects_created_subscribed: false,
            objects_destroyed_subscribed: false,
            services_created_subscribed: false,
            services_destroyed_subscribed: false,
            subscriptions: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, cookie: ObjectCookie) {
        let unique = self.objects.insert(cookie);
        debug_assert!(unique);
    }

    pub fn remove_object(&mut self, cookie: ObjectCookie) {
        let contained = self.objects.remove(&cookie);
        debug_assert!(contained);
    }

    pub fn objects<'a>(&'a self) -> impl Iterator<Item = ObjectCookie> + 'a {
        self.objects.iter().copied()
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

    pub fn set_objects_destroyed_subscribed(&mut self, subscribed: bool) {
        self.objects_destroyed_subscribed = subscribed;
    }

    pub fn objects_destroyed_subscribed(&self) -> bool {
        self.objects_destroyed_subscribed
    }

    pub fn set_services_created_subscribed(&mut self, subscribed: bool) {
        self.services_created_subscribed = subscribed;
    }

    pub fn services_created_subscribed(&self) -> bool {
        self.services_created_subscribed
    }

    pub fn set_services_destroyed_subscribed(&mut self, subscribed: bool) {
        self.services_destroyed_subscribed = subscribed;
    }

    pub fn services_destroyed_subscribed(&self) -> bool {
        self.services_destroyed_subscribed
    }

    pub fn add_subscription(&mut self, svc_cookie: ServiceCookie, id: u32) {
        self.subscriptions.entry(svc_cookie).or_default().insert(id);
    }

    pub fn remove_subscription(&mut self, svc_cookie: ServiceCookie, id: u32) {
        if let Entry::Occupied(mut subs) = self.subscriptions.entry(svc_cookie) {
            subs.get_mut().remove(&id);
            if subs.get().is_empty() {
                subs.remove();
            }
        }
    }

    pub fn remove_all_subscriptions(&mut self, svc_cookie: ServiceCookie) {
        self.subscriptions.remove(&svc_cookie);
    }

    pub fn subscriptions<'a>(&'a self) -> impl Iterator<Item = (ServiceCookie, u32)> + 'a {
        self.subscriptions
            .iter()
            .flat_map(|(&c, ids)| ids.iter().map(move |&id| (c, id)))
    }

    pub fn is_subscribed_to(&self, svc_cookie: ServiceCookie, id: u32) -> bool {
        self.subscriptions
            .get(&svc_cookie)
            .map(|s| s.contains(&id))
            .unwrap_or(false)
    }
}
