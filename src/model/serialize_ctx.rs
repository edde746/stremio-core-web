use wasm_bindgen::JsValue;

use stremio_core::models::ctx::Ctx;

pub fn serialize_ctx(ctx: &Ctx) -> JsValue {
    JsValue::from_serde(&model::Ctx::from(ctx)).unwrap()
}

mod model {
    use std::collections::HashMap;

    use serde::Serialize;

    use chrono::{DateTime, Utc};

    use stremio_core::types::{
        notifications::NotificationItem, profile::Profile, resource::MetaItemId,
    };

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Ctx<'a> {
        /// keep the original Profile model inside.
        pub profile: &'a Profile,
        pub notifications: Notifications<'a>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Notifications<'a> {
        /// Override the notifications to simplify the mapping
        pub items: HashMap<MetaItemId, Vec<&'a NotificationItem>>,
        pub last_updated: Option<DateTime<Utc>>,
        pub created: DateTime<Utc>,
    }

    impl<'a> From<&'a stremio_core::models::ctx::Ctx> for Ctx<'a> {
        fn from(ctx: &'a stremio_core::models::ctx::Ctx) -> Self {
            Self {
                profile: &ctx.profile,
                notifications: Notifications {
                    items: ctx
                        .notifications
                        .items
                        .iter()
                        .map(|(meta_id, notifications)| {
                            (meta_id.to_owned(), notifications.values().collect())
                        })
                        .collect(),
                    last_updated: ctx.notifications.last_updated,
                    created: ctx.notifications.created,
                },
            }
        }
    }
}
