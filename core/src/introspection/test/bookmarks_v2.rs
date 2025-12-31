use super::super::{DynIntrospectable, Introspectable, LexicalId, References, ir};
use crate::ServiceUuid;
use uuid::uuid;

pub(crate) struct Bookmarks;

impl Introspectable for Bookmarks {
    fn layout() -> ir::LayoutIr {
        ir::ServiceIr::builder(
            "bookmarks_v2",
            "Bookmarks",
            ServiceUuid(uuid!("35660342-8ecb-4101-903a-d1ba49d66f29")),
            2,
        )
        .function(
            ir::FunctionIr::builder(1, "get")
                .ok(Vec::<Bookmark>::lexical_id())
                .finish(),
        )
        .function(
            ir::FunctionIr::builder(4, "get_v2")
                .args(BookmarksGetV2Args::lexical_id())
                .ok(Vec::<Bookmark>::lexical_id())
                .err(Error::lexical_id())
                .finish(),
        )
        .function(
            ir::FunctionIr::builder(2, "add")
                .args(Bookmark::lexical_id())
                .err(Error::lexical_id())
                .finish(),
        )
        .function(
            ir::FunctionIr::builder(3, "remove")
                .args(String::lexical_id())
                .err(Error::lexical_id())
                .finish(),
        )
        .function(
            ir::FunctionIr::builder(5, "remove_v2")
                .args(BookmarksRemoveV2Args::lexical_id())
                .err(Error::lexical_id())
                .finish(),
        )
        .function(
            ir::FunctionIr::builder(6, "get_groups")
                .ok(Vec::<Option<String>>::lexical_id())
                .finish(),
        )
        .event(
            ir::EventIr::builder(1, "added")
                .event_type(Bookmark::lexical_id())
                .finish(),
        )
        .event(
            ir::EventIr::builder(3, "added_v2")
                .event_type(Bookmark::lexical_id())
                .finish(),
        )
        .event(
            ir::EventIr::builder(2, "removed")
                .event_type(Bookmark::lexical_id())
                .finish(),
        )
        .event(
            ir::EventIr::builder(4, "removed_v2")
                .event_type(Bookmark::lexical_id())
                .finish(),
        )
        .function_fallback(ir::FunctionFallbackIr::builder("unknown_function").finish())
        .event_fallback(ir::EventFallbackIr::builder("unknown_event").finish())
        .finish()
        .into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::service("bookmarks_v2", "Bookmarks")
    }

    fn add_references(references: &mut References) {
        let types = [
            DynIntrospectable::new::<Error>(),
            DynIntrospectable::new::<Bookmark>(),
            DynIntrospectable::new::<BookmarksGetV2Args>(),
            DynIntrospectable::new::<BookmarksRemoveV2Args>(),
            DynIntrospectable::new::<Vec<Bookmark>>(),
            DynIntrospectable::new::<Vec<Option<String>>>(),
            DynIntrospectable::new::<String>(),
        ];

        references.extend(types);
    }
}

pub(crate) struct BookmarksGetV2Args;

impl Introspectable for BookmarksGetV2Args {
    fn layout() -> ir::LayoutIr {
        ir::StructIr::builder("bookmarks_v2", "BookmarksGetV2Args")
            .field(ir::FieldIr::builder(1, "group", false, String::lexical_id()).finish())
            .fallback(ir::StructFallbackIr::builder("unknown_fields").finish())
            .finish()
            .into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::custom("bookmarks_v2", "BookmarksGetV2Args")
    }

    fn add_references(references: &mut References) {
        let types = [DynIntrospectable::new::<String>()];

        references.extend(types);
    }
}

pub(crate) struct BookmarksRemoveV2Args;

impl Introspectable for BookmarksRemoveV2Args {
    fn layout() -> ir::LayoutIr {
        ir::StructIr::builder("bookmarks_v2", "BookmarksRemoveV2Args")
            .field(ir::FieldIr::builder(1, "name", true, String::lexical_id()).finish())
            .field(ir::FieldIr::builder(2, "group", false, String::lexical_id()).finish())
            .fallback(ir::StructFallbackIr::builder("unknown_fields").finish())
            .finish()
            .into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::custom("bookmarks_v2", "BookmarksRemoveV2Args")
    }

    fn add_references(references: &mut References) {
        let types = [
            DynIntrospectable::new::<String>(),
            DynIntrospectable::new::<String>(),
        ];

        references.extend(types);
    }
}

pub(crate) struct Bookmark;

impl Introspectable for Bookmark {
    fn layout() -> ir::LayoutIr {
        ir::StructIr::builder("bookmarks_v2", "Bookmark")
            .field(ir::FieldIr::builder(1, "name", true, String::lexical_id()).finish())
            .field(ir::FieldIr::builder(2, "url", true, String::lexical_id()).finish())
            .field(ir::FieldIr::builder(3, "group", false, String::lexical_id()).finish())
            .fallback(ir::StructFallbackIr::builder("unknown_fields").finish())
            .finish()
            .into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::custom("bookmarks_v2", "Bookmark")
    }

    fn add_references(references: &mut References) {
        let types = [
            DynIntrospectable::new::<String>(),
            DynIntrospectable::new::<String>(),
            DynIntrospectable::new::<String>(),
        ];

        references.extend(types);
    }
}

pub(crate) enum Error {}

impl Introspectable for Error {
    fn layout() -> ir::LayoutIr {
        ir::EnumIr::builder("bookmarks_v2", "Error")
            .variant(ir::VariantIr::builder(1, "InvalidName").finish())
            .variant(ir::VariantIr::builder(2, "DuplicateName").finish())
            .variant(ir::VariantIr::builder(3, "InvalidUrl").finish())
            .variant(ir::VariantIr::builder(4, "UnknownFields").finish())
            .variant(ir::VariantIr::builder(5, "InvalidGroup").finish())
            .fallback(ir::EnumFallbackIr::builder("Unknown").finish())
            .finish()
            .into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::custom("bookmarks_v2", "Error")
    }

    fn add_references(_references: &mut References) {}
}
