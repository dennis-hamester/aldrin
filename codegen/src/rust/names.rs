use crate::util;

/// Returns the name for a functions's enum variant.
pub fn function_variant(func: &str) -> String {
    util::to_camel_case(func)
}

/// Returns the name for an inline struct or enum used as a functions's `args`.
pub fn function_args(service: &str, func: &str) -> String {
    let func = util::to_camel_case(func);
    format!("{service}{func}Args")
}

/// Returns the name for an inline struct or enum used as a functions's `ok`.
pub fn function_ok(service: &str, func: &str) -> String {
    let func = util::to_camel_case(func);
    format!("{service}{func}Ok")
}

/// Returns the name for an inline struct or enum used as a functions's `err`.
pub fn function_err(service: &str, func: &str) -> String {
    let func = util::to_camel_case(func);
    format!("{service}{func}Error")
}

/// Returns the name of the function call that takes an argument by value.
pub fn call_val(func: &str) -> String {
    format!("{func}_val")
}

/// Returns the name of the function call that takes an argument by reference.
pub fn call_ref(func: &str) -> String {
    format!("{func}_ref")
}

/// Returns the name for an event's enum variant.
pub fn event_variant(event: &str) -> String {
    util::to_camel_case(event)
}

/// Returns the name for an inline struct or enum used as an event's argument.
pub fn event_args(service: &str, event: &str) -> String {
    let event = util::to_camel_case(event);
    format!("{service}{event}Args")
}

/// Returns the name of the event emitter that takes an argument by value.
pub fn emit_val(event: &str) -> String {
    format!("{event}_val")
}

/// Returns the name of the event emitter that takes an argument by reference.
pub fn emit_ref(event: &str) -> String {
    format!("{event}_ref")
}

/// Returns the name of event's subscribe function.
pub fn subscribe(event: &str) -> String {
    format!("subscribe_{event}")
}

/// Returns the name of event's unsubscribe function.
pub fn unsubscribe(event: &str) -> String {
    format!("unsubscribe_{event}")
}

/// Returns the name of a service's proxy type.
pub fn service_proxy(service: &str) -> String {
    format!("{service}Proxy")
}

/// Returns the name of a service's event type.
pub fn service_event(service: &str) -> String {
    format!("{service}Event")
}

/// Returns the name of a service's local event handler trait.
pub fn service_local_event_handler(service: &str) -> String {
    format!("Local{service}EventHandler")
}

/// Returns the name of a service's event handler trait.
pub fn service_event_handler(service: &str) -> String {
    format!("{service}EventHandler")
}

/// Returns the name of a service's call type.
pub fn service_call(service: &str) -> String {
    format!("{service}Call")
}

/// Returns the name of a service's local call handler trait.
pub fn service_local_call_handler(service: &str) -> String {
    format!("Local{service}CallHandler")
}

/// Returns the name of a service's call handler trait.
pub fn service_call_handler(service: &str) -> String {
    format!("{service}CallHandler")
}

/// Returns the name of a service's introspection type.
pub fn service_introspection(service: &str) -> String {
    format!("{service}Introspection")
}

/// Returns the default name for a ref type.
pub fn default_ref_type(ty: &str) -> String {
    format!("{ty}Ref")
}

/// Returns the name of the constructor function in a ref type for an enum's variant.
pub fn enum_ref_type_ctor(variant: &str) -> String {
    util::to_snake_case(variant)
}

pub fn register_introspection(schema: &str) -> String {
    format!("register_introspection_{schema}")
}
