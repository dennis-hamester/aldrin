use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "filter", deny_unknown_fields)]
pub enum BusListenerFilter {
    AnyObject,
    SpecificObject { object: UuidRef },
    AnyObjectAnyService,
    SpecificObjectAnyService { object: UuidRef },
    AnyObjectSpecificService { service: UuidRef },
    SpecificObjectSpecificService { object: UuidRef, service: UuidRef },
}

impl BusListenerFilter {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::BusListenerFilter> {
        match self {
            Self::AnyObject => Ok(message::BusListenerFilter::any_object()),

            Self::SpecificObject { object } => {
                let object = object.get(ctx)?.into();
                Ok(message::BusListenerFilter::object(object))
            }

            Self::AnyObjectAnyService => Ok(message::BusListenerFilter::any_service_any_object()),

            Self::SpecificObjectAnyService { object } => {
                let object = object.get(ctx)?.into();

                Ok(message::BusListenerFilter::any_service_specific_object(
                    object,
                ))
            }

            Self::AnyObjectSpecificService { service } => {
                let service = service.get(ctx)?.into();

                Ok(message::BusListenerFilter::specific_service_any_object(
                    service,
                ))
            }

            Self::SpecificObjectSpecificService { object, service } => {
                let object = object.get(ctx)?.into();
                let service = service.get(ctx)?.into();

                Ok(message::BusListenerFilter::specific_service_and_object(
                    service, object,
                ))
            }
        }
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::AnyObject, Self::AnyObject)
            | (Self::AnyObjectAnyService, Self::AnyObjectAnyService) => Ok(true),

            (
                Self::SpecificObject { object: object1 },
                Self::SpecificObject { object: object2 },
            ) => object1.matches(object2, ctx),

            (
                Self::SpecificObjectAnyService { object: object1 },
                Self::SpecificObjectAnyService { object: object2 },
            ) => object1.matches(object2, ctx),

            (
                Self::AnyObjectSpecificService { service: service1 },
                Self::AnyObjectSpecificService { service: service2 },
            ) => service1.matches(service2, ctx),

            (
                Self::SpecificObjectSpecificService {
                    object: object1,
                    service: service1,
                },
                Self::SpecificObjectSpecificService {
                    object: object2,
                    service: service2,
                },
            ) => {
                let res = object1.matches(object2, ctx)? && service1.matches(service2, ctx)?;
                Ok(res)
            }

            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (Self::AnyObject, Self::AnyObject)
            | (Self::AnyObjectAnyService, Self::AnyObjectAnyService) => Ok(()),

            (
                Self::SpecificObject { object: object1 },
                Self::SpecificObject { object: object2 },
            ) => object1.update_context(object2, ctx),

            (
                Self::SpecificObjectAnyService { object: object1 },
                Self::SpecificObjectAnyService { object: object2 },
            ) => object1.update_context(object2, ctx),

            (
                Self::AnyObjectSpecificService { service: service1 },
                Self::AnyObjectSpecificService { service: service2 },
            ) => service1.update_context(service2, ctx),

            (
                Self::SpecificObjectSpecificService {
                    object: object1,
                    service: service1,
                },
                Self::SpecificObjectSpecificService {
                    object: object2,
                    service: service2,
                },
            ) => {
                object1.update_context(object2, ctx)?;
                service1.update_context(service2, ctx)?;

                Ok(())
            }

            _ => unreachable!(),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::AnyObject => Ok(Self::AnyObject),

            Self::SpecificObject { object } => {
                let object = object.apply_context(ctx)?;
                Ok(Self::SpecificObject { object })
            }

            Self::AnyObjectAnyService => Ok(Self::AnyObjectAnyService),

            Self::SpecificObjectAnyService { object } => {
                let object = object.apply_context(ctx)?;
                Ok(Self::SpecificObjectAnyService { object })
            }

            Self::AnyObjectSpecificService { service } => {
                let service = service.apply_context(ctx)?;
                Ok(Self::AnyObjectSpecificService { service })
            }

            Self::SpecificObjectSpecificService { object, service } => {
                let object = object.apply_context(ctx)?;
                let service = service.apply_context(ctx)?;
                Ok(Self::SpecificObjectSpecificService { object, service })
            }
        }
    }
}

impl From<message::BusListenerFilter> for BusListenerFilter {
    fn from(filter: message::BusListenerFilter) -> Self {
        match filter {
            message::BusListenerFilter::Object(None) => Self::AnyObject,

            message::BusListenerFilter::Object(Some(object)) => Self::SpecificObject {
                object: object.into(),
            },

            message::BusListenerFilter::Service(message::BusListenerServiceFilter {
                object: None,
                service: None,
            }) => Self::AnyObjectAnyService,

            message::BusListenerFilter::Service(message::BusListenerServiceFilter {
                object: Some(object),
                service: None,
            }) => Self::SpecificObjectAnyService {
                object: object.into(),
            },

            message::BusListenerFilter::Service(message::BusListenerServiceFilter {
                object: None,
                service: Some(service),
            }) => Self::AnyObjectSpecificService {
                service: service.into(),
            },

            message::BusListenerFilter::Service(message::BusListenerServiceFilter {
                object: Some(object),
                service: Some(service),
            }) => Self::SpecificObjectSpecificService {
                object: object.into(),
                service: service.into(),
            },
        }
    }
}
