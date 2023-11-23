use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use aldrin_core::{BusEvent, BusListenerCookie, ObjectId, ServiceId};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "event")]
pub enum EmitBusEvent {
    #[serde(rename_all = "kebab-case")]
    ObjectCreated {
        #[serde(skip_serializing_if = "Option::is_none")]
        cookie: Option<UuidRef>,

        object_uuid: UuidRef,
        object_cookie: UuidRef,
    },

    #[serde(rename_all = "kebab-case")]
    ObjectDestroyed {
        #[serde(skip_serializing_if = "Option::is_none")]
        cookie: Option<UuidRef>,

        object_uuid: UuidRef,
        object_cookie: UuidRef,
    },

    #[serde(rename_all = "kebab-case")]
    ServiceCreated {
        #[serde(skip_serializing_if = "Option::is_none")]
        cookie: Option<UuidRef>,

        object_uuid: UuidRef,
        object_cookie: UuidRef,
        service_uuid: UuidRef,
        service_cookie: UuidRef,
    },

    #[serde(rename_all = "kebab-case")]
    ServiceDestroyed {
        #[serde(skip_serializing_if = "Option::is_none")]
        cookie: Option<UuidRef>,

        object_uuid: UuidRef,
        object_cookie: UuidRef,
        service_uuid: UuidRef,
        service_cookie: UuidRef,
    },
}

impl EmitBusEvent {
    pub fn to_core(&self, ctx: &Context) -> Result<message::EmitBusEvent> {
        match self {
            Self::ObjectCreated {
                cookie,
                object_uuid,
                object_cookie,
            } => {
                let cookie = cookie.as_ref().map(|cookie| cookie.get(ctx)).transpose()?;
                let object_uuid = object_uuid.get(ctx)?.into();
                let object_cookie = object_cookie.get(ctx)?.into();

                Ok(message::EmitBusEvent {
                    cookie: cookie.map(BusListenerCookie::from),
                    event: BusEvent::ObjectCreated(ObjectId::new(object_uuid, object_cookie)),
                })
            }

            Self::ObjectDestroyed {
                cookie,
                object_uuid,
                object_cookie,
            } => {
                let cookie = cookie.as_ref().map(|cookie| cookie.get(ctx)).transpose()?;
                let object_uuid = object_uuid.get(ctx)?.into();
                let object_cookie = object_cookie.get(ctx)?.into();

                Ok(message::EmitBusEvent {
                    cookie: cookie.map(BusListenerCookie::from),
                    event: BusEvent::ObjectDestroyed(ObjectId::new(object_uuid, object_cookie)),
                })
            }

            Self::ServiceCreated {
                cookie,
                object_uuid,
                object_cookie,
                service_uuid,
                service_cookie,
            } => {
                let cookie = cookie.as_ref().map(|cookie| cookie.get(ctx)).transpose()?;
                let object_uuid = object_uuid.get(ctx)?.into();
                let object_cookie = object_cookie.get(ctx)?.into();
                let service_uuid = service_uuid.get(ctx)?.into();
                let service_cookie = service_cookie.get(ctx)?.into();

                Ok(message::EmitBusEvent {
                    cookie: cookie.map(BusListenerCookie::from),
                    event: BusEvent::ServiceCreated(ServiceId::new(
                        ObjectId::new(object_uuid, object_cookie),
                        service_uuid,
                        service_cookie,
                    )),
                })
            }

            Self::ServiceDestroyed {
                cookie,
                object_uuid,
                object_cookie,
                service_uuid,
                service_cookie,
            } => {
                let cookie = cookie.as_ref().map(|cookie| cookie.get(ctx)).transpose()?;
                let object_uuid = object_uuid.get(ctx)?.into();
                let object_cookie = object_cookie.get(ctx)?.into();
                let service_uuid = service_uuid.get(ctx)?.into();
                let service_cookie = service_cookie.get(ctx)?.into();

                Ok(message::EmitBusEvent {
                    cookie: cookie.map(BusListenerCookie::from),
                    event: BusEvent::ServiceDestroyed(ServiceId::new(
                        ObjectId::new(object_uuid, object_cookie),
                        service_uuid,
                        service_cookie,
                    )),
                })
            }
        }
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (
                Self::ObjectCreated {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                },
                Self::ObjectCreated {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                },
            ) => {
                let res = match (cookie1, cookie2) {
                    (Some(cookie1), Some(cookie2)) => cookie1.matches(cookie2, ctx)?,
                    (Some(_), None) | (None, Some(_)) => return Ok(false),
                    (None, None) => true,
                };

                let res = res
                    && object_uuid1.matches(object_uuid2, ctx)?
                    && object_cookie1.matches(object_cookie2, ctx)?;

                Ok(res)
            }

            (
                Self::ObjectDestroyed {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                },
                Self::ObjectDestroyed {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                },
            ) => {
                let res = match (cookie1, cookie2) {
                    (Some(cookie1), Some(cookie2)) => cookie1.matches(cookie2, ctx)?,
                    (Some(_), None) | (None, Some(_)) => return Ok(false),
                    (None, None) => true,
                };

                let res = res
                    && object_uuid1.matches(object_uuid2, ctx)?
                    && object_cookie1.matches(object_cookie2, ctx)?;

                Ok(res)
            }

            (
                Self::ServiceCreated {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                    service_uuid: service_uuid1,
                    service_cookie: service_cookie1,
                },
                Self::ServiceCreated {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                    service_uuid: service_uuid2,
                    service_cookie: service_cookie2,
                },
            ) => {
                let res = match (cookie1, cookie2) {
                    (Some(cookie1), Some(cookie2)) => cookie1.matches(cookie2, ctx)?,
                    (Some(_), None) | (None, Some(_)) => return Ok(false),
                    (None, None) => true,
                };

                let res = res
                    && object_uuid1.matches(object_uuid2, ctx)?
                    && object_cookie1.matches(object_cookie2, ctx)?
                    && service_uuid1.matches(service_uuid2, ctx)?
                    && service_cookie1.matches(service_cookie2, ctx)?;

                Ok(res)
            }

            (
                Self::ServiceDestroyed {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                    service_uuid: service_uuid1,
                    service_cookie: service_cookie1,
                },
                Self::ServiceDestroyed {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                    service_uuid: service_uuid2,
                    service_cookie: service_cookie2,
                },
            ) => {
                let res = match (cookie1, cookie2) {
                    (Some(cookie1), Some(cookie2)) => cookie1.matches(cookie2, ctx)?,
                    (Some(_), None) | (None, Some(_)) => return Ok(false),
                    (None, None) => true,
                };

                let res = res
                    && object_uuid1.matches(object_uuid2, ctx)?
                    && object_cookie1.matches(object_cookie2, ctx)?
                    && service_uuid1.matches(service_uuid2, ctx)?
                    && service_cookie1.matches(service_cookie2, ctx)?;

                Ok(res)
            }

            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (
                Self::ObjectCreated {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                },
                Self::ObjectCreated {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                },
            ) => {
                if let (Some(cookie1), Some(cookie2)) = (cookie1, cookie2) {
                    cookie1.update_context(cookie2, ctx)?;
                }

                object_uuid1.update_context(object_uuid2, ctx)?;
                object_cookie1.update_context(object_cookie2, ctx)?;

                Ok(())
            }

            (
                Self::ObjectDestroyed {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                },
                Self::ObjectDestroyed {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                },
            ) => {
                if let (Some(cookie1), Some(cookie2)) = (cookie1, cookie2) {
                    cookie1.update_context(cookie2, ctx)?;
                }

                object_uuid1.update_context(object_uuid2, ctx)?;
                object_cookie1.update_context(object_cookie2, ctx)?;

                Ok(())
            }

            (
                Self::ServiceCreated {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                    service_uuid: service_uuid1,
                    service_cookie: service_cookie1,
                },
                Self::ServiceCreated {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                    service_uuid: service_uuid2,
                    service_cookie: service_cookie2,
                },
            ) => {
                if let (Some(cookie1), Some(cookie2)) = (cookie1, cookie2) {
                    cookie1.update_context(cookie2, ctx)?;
                }

                object_uuid1.update_context(object_uuid2, ctx)?;
                object_cookie1.update_context(object_cookie2, ctx)?;
                service_uuid1.update_context(service_uuid2, ctx)?;
                service_cookie1.update_context(service_cookie2, ctx)?;

                Ok(())
            }

            (
                Self::ServiceDestroyed {
                    cookie: cookie1,
                    object_uuid: object_uuid1,
                    object_cookie: object_cookie1,
                    service_uuid: service_uuid1,
                    service_cookie: service_cookie1,
                },
                Self::ServiceDestroyed {
                    cookie: cookie2,
                    object_uuid: object_uuid2,
                    object_cookie: object_cookie2,
                    service_uuid: service_uuid2,
                    service_cookie: service_cookie2,
                },
            ) => {
                if let (Some(cookie1), Some(cookie2)) = (cookie1, cookie2) {
                    cookie1.update_context(cookie2, ctx)?;
                }

                object_uuid1.update_context(object_uuid2, ctx)?;
                object_cookie1.update_context(object_cookie2, ctx)?;
                service_uuid1.update_context(service_uuid2, ctx)?;
                service_cookie1.update_context(service_cookie2, ctx)?;

                Ok(())
            }

            _ => unreachable!(),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::ObjectCreated {
                cookie,
                object_uuid,
                object_cookie,
            } => {
                let cookie = cookie
                    .as_ref()
                    .map(|cookie| cookie.apply_context(ctx))
                    .transpose()?;
                let object_uuid = object_uuid.apply_context(ctx)?;
                let object_cookie = object_cookie.apply_context(ctx)?;

                Ok(Self::ObjectCreated {
                    cookie,
                    object_uuid,
                    object_cookie,
                })
            }

            Self::ObjectDestroyed {
                cookie,
                object_uuid,
                object_cookie,
            } => {
                let cookie = cookie
                    .as_ref()
                    .map(|cookie| cookie.apply_context(ctx))
                    .transpose()?;
                let object_uuid = object_uuid.apply_context(ctx)?;
                let object_cookie = object_cookie.apply_context(ctx)?;

                Ok(Self::ObjectDestroyed {
                    cookie,
                    object_uuid,
                    object_cookie,
                })
            }

            Self::ServiceCreated {
                cookie,
                object_uuid,
                object_cookie,
                service_uuid,
                service_cookie,
            } => {
                let cookie = cookie
                    .as_ref()
                    .map(|cookie| cookie.apply_context(ctx))
                    .transpose()?;
                let object_uuid = object_uuid.apply_context(ctx)?;
                let object_cookie = object_cookie.apply_context(ctx)?;
                let service_uuid = service_uuid.apply_context(ctx)?;
                let service_cookie = service_cookie.apply_context(ctx)?;

                Ok(Self::ServiceCreated {
                    cookie,
                    object_uuid,
                    object_cookie,
                    service_uuid,
                    service_cookie,
                })
            }

            Self::ServiceDestroyed {
                cookie,
                object_uuid,
                object_cookie,
                service_uuid,
                service_cookie,
            } => {
                let cookie = cookie
                    .as_ref()
                    .map(|cookie| cookie.apply_context(ctx))
                    .transpose()?;
                let object_uuid = object_uuid.apply_context(ctx)?;
                let object_cookie = object_cookie.apply_context(ctx)?;
                let service_uuid = service_uuid.apply_context(ctx)?;
                let service_cookie = service_cookie.apply_context(ctx)?;

                Ok(Self::ServiceDestroyed {
                    cookie,
                    object_uuid,
                    object_cookie,
                    service_uuid,
                    service_cookie,
                })
            }
        }
    }
}

impl TryFrom<message::EmitBusEvent> for EmitBusEvent {
    type Error = Error;

    fn try_from(msg: message::EmitBusEvent) -> Result<Self> {
        match msg {
            message::EmitBusEvent {
                cookie,
                event: BusEvent::ObjectCreated(object),
            } => Ok(Self::ObjectCreated {
                cookie: cookie.map(UuidRef::from),
                object_uuid: object.uuid.into(),
                object_cookie: object.cookie.into(),
            }),

            message::EmitBusEvent {
                cookie,
                event: BusEvent::ObjectDestroyed(object),
            } => Ok(Self::ObjectDestroyed {
                cookie: cookie.map(UuidRef::from),
                object_uuid: object.uuid.into(),
                object_cookie: object.cookie.into(),
            }),

            message::EmitBusEvent {
                cookie,
                event: BusEvent::ServiceCreated(service),
            } => Ok(Self::ServiceCreated {
                cookie: cookie.map(UuidRef::from),
                object_uuid: service.object_id.uuid.into(),
                object_cookie: service.object_id.cookie.into(),
                service_uuid: service.uuid.into(),
                service_cookie: service.cookie.into(),
            }),

            message::EmitBusEvent {
                cookie,
                event: BusEvent::ServiceDestroyed(service),
            } => Ok(Self::ServiceDestroyed {
                cookie: cookie.map(UuidRef::from),
                object_uuid: service.object_id.uuid.into(),
                object_cookie: service.object_id.cookie.into(),
                service_uuid: service.uuid.into(),
                service_cookie: service.cookie.into(),
            }),
        }
    }
}
