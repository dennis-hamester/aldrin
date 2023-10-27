use crate::context::Context;
use crate::uuid_ref::UuidRef;
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
    pub fn to_proto(&self, ctx: &Context) -> Result<aldrin_proto::BusListenerFilter> {
        match self {
            Self::AnyObject => Ok(aldrin_proto::BusListenerFilter::any_object()),

            Self::SpecificObject { object } => {
                let object = object.get(ctx)?.into();
                Ok(aldrin_proto::BusListenerFilter::object(object))
            }

            Self::AnyObjectAnyService => {
                Ok(aldrin_proto::BusListenerFilter::any_object_any_service())
            }

            Self::SpecificObjectAnyService { object } => {
                let object = object.get(ctx)?.into();

                Ok(aldrin_proto::BusListenerFilter::specific_object_any_service(object))
            }

            Self::AnyObjectSpecificService { service } => {
                let service = service.get(ctx)?.into();

                Ok(aldrin_proto::BusListenerFilter::any_object_specific_service(service))
            }

            Self::SpecificObjectSpecificService { object, service } => {
                let object = object.get(ctx)?.into();
                let service = service.get(ctx)?.into();

                Ok(aldrin_proto::BusListenerFilter::specific_object_and_service(object, service))
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

impl From<aldrin_proto::BusListenerFilter> for BusListenerFilter {
    fn from(filter: aldrin_proto::BusListenerFilter) -> Self {
        match filter {
            aldrin_proto::BusListenerFilter::Object(None) => Self::AnyObject,

            aldrin_proto::BusListenerFilter::Object(Some(object)) => Self::SpecificObject {
                object: object.into(),
            },

            aldrin_proto::BusListenerFilter::Service(aldrin_proto::BusListenerServiceFilter {
                object: None,
                service: None,
            }) => Self::AnyObjectAnyService,

            aldrin_proto::BusListenerFilter::Service(aldrin_proto::BusListenerServiceFilter {
                object: Some(object),
                service: None,
            }) => Self::SpecificObjectAnyService {
                object: object.into(),
            },

            aldrin_proto::BusListenerFilter::Service(aldrin_proto::BusListenerServiceFilter {
                object: None,
                service: Some(service),
            }) => Self::AnyObjectSpecificService {
                service: service.into(),
            },

            aldrin_proto::BusListenerFilter::Service(aldrin_proto::BusListenerServiceFilter {
                object: Some(object),
                service: Some(service),
            }) => Self::SpecificObjectSpecificService {
                object: object.into(),
                service: service.into(),
            },
        }
    }
}
