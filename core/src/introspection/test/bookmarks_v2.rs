use super::super::{ir, DynIntrospectable, Introspectable, LexicalId, References};
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
        .function(1, "get", None, Some(Vec::<Bookmark>::lexical_id()), None)
        .function(
            4,
            "get_v2",
            Some(BookmarksGetV2Args::lexical_id()),
            Some(Vec::<Bookmark>::lexical_id()),
            Some(Error::lexical_id()),
        )
        .function(
            2,
            "add",
            Some(Bookmark::lexical_id()),
            None,
            Some(Error::lexical_id()),
        )
        .function(
            3,
            "remove",
            Some(String::lexical_id()),
            None,
            Some(Error::lexical_id()),
        )
        .function(
            5,
            "remove_v2",
            Some(BookmarksRemoveV2Args::lexical_id()),
            None,
            Some(Error::lexical_id()),
        )
        .function(
            6,
            "get_groups",
            None,
            Some(Vec::<Option<String>>::lexical_id()),
            None,
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
            .unit_variant(1, "InvalidName")
            .unit_variant(2, "DuplicateName")
            .unit_variant(3, "InvalidUrl")
            .unit_variant(4, "UnknownFields")
            .unit_variant(5, "InvalidGroup")
            .fallback(ir::EnumFallbackIr::builder("Unknown").finish())
            .finish()
            .into()
    }

    fn lexical_id() -> LexicalId {
        LexicalId::custom("bookmarks_v2", "Error")
    }

    fn add_references(_references: &mut References) {}
}
